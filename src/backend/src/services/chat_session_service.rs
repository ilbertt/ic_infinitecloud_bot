use frankenstein::{CallbackQuery, MaybeInaccessibleMessage, Message};

use crate::{
    custom_print,
    repositories::{
        with_clear_action_on_error, ChatId, ChatSession, ChatSessionAction, ChatSessionRepository,
        ChatSessionRepositoryImpl, ChatSessionWaitReply, Command, FileSystem, FileSystemNode,
        FilesystemRepositoryImpl, KeyboardDirectoryBuilder, MessageId,
    },
    utils::{
        filesystem::root_path,
        messages::{
            ask_directory_name_message, ask_file_name_message, ask_rename_file_message,
            back_inline_keyboard, create_file_message, created_directory_success_message,
            created_file_success_message, explorer_file_message, explorer_message, help_message,
            info_message, mkdir_message, move_file_select_destination_message,
            move_file_select_file_message, moved_file_success_message, rename_file_message,
            renamed_file_success_message, start_message, COMING_SOON_TEXT,
        },
        MessageParams, TG_FILE_MIME_TYPE_PREFIX,
    },
};

use super::{FilesystemService, FilesystemServiceImpl};

pub trait ChatSessionService {
    fn get_or_create_chat_session(&self, chat_id: &ChatId) -> ChatSession;

    fn update_chat_session(&self, chat_id: ChatId, chat_session: ChatSession);

    fn get_chat_sessions_count(&self) -> u32;

    fn handle_update_content_message(
        &self,
        chat_id: ChatId,
        msg: Message,
    ) -> Result<MessageParams, String>;

    fn handle_update_content_callback_query(
        &self,
        chat_id: ChatId,
        query: CallbackQuery,
    ) -> Result<MessageParams, String>;
}

pub struct ChatSessionServiceImpl<T: ChatSessionRepository, F: FilesystemService> {
    chat_session_repository: T,
    filesystem_service: F,
}

impl Default
    for ChatSessionServiceImpl<
        ChatSessionRepositoryImpl,
        FilesystemServiceImpl<FilesystemRepositoryImpl>,
    >
{
    fn default() -> Self {
        Self::new(
            ChatSessionRepositoryImpl::default(),
            FilesystemServiceImpl::default(),
        )
    }
}

impl<T: ChatSessionRepository, F: FilesystemService> ChatSessionService
    for ChatSessionServiceImpl<T, F>
{
    fn get_or_create_chat_session(&self, chat_id: &ChatId) -> ChatSession {
        match self
            .chat_session_repository
            .get_chat_session_by_chat_id(chat_id)
        {
            Some(chat_session) => chat_session,
            None => {
                let chat_session = ChatSession::default();
                self.chat_session_repository
                    .set_chat_session_by_chat_id(chat_id.clone(), chat_session.clone());
                chat_session
            }
        }
    }

    fn update_chat_session(&self, chat_id: ChatId, chat_session: ChatSession) {
        self.chat_session_repository
            .set_chat_session_by_chat_id(chat_id, chat_session);
    }

    fn get_chat_sessions_count(&self) -> u32 {
        self.chat_session_repository.get_chat_session_count() as u32
    }

    fn handle_update_content_message(
        &self,
        chat_id: ChatId,
        msg: Message,
    ) -> Result<MessageParams, String> {
        let mut fs = self.filesystem_service.get_or_create_filesystem(&chat_id);
        let mut chat_session = self.get_or_create_chat_session(&chat_id);

        let from_user = msg.clone().from;

        let res = with_clear_action_on_error(&mut chat_session, |cs| {
            let current_path = cs.current_path().clone();
            custom_print!(
                        "UpdateContent::Message: chat_id: {:?}, current_path: {:?}, current_action: {:?}, message.text: {:?}",
                        chat_id, current_path, cs.action(), msg.text
                    );

            match Command::try_from(msg.clone()) {
                Ok(command) => {
                    // when receiving a command, we want to reset the chat session
                    cs.reset();

                    let mut send_message_params = MessageParams::new_send(chat_id.clone());

                    match command {
                        Command::Start => {
                            send_message_params
                                .set_text(start_message(from_user.map(|user| user.first_name)));
                        }
                        Command::Help => send_message_params.set_text(help_message()),
                        Command::Info => send_message_params.set_text(info_message()),
                        Command::MkDir => {
                            cs.set_action(ChatSessionAction::MkDir(None));

                            send_message_params.set_text(mkdir_message(cs.current_path_string()));

                            let keyboard = KeyboardDirectoryBuilder::new(&fs, cs.current_path())?
                                .with_current_dir_button()
                                .build();
                            send_message_params.set_inline_keyboard_markup(keyboard);
                        }
                        Command::Explorer => {
                            cs.set_action(ChatSessionAction::Explorer);

                            send_message_params
                                .set_text(explorer_message(cs.current_path_string()));

                            let keyboard = KeyboardDirectoryBuilder::new(&fs, cs.current_path())?
                                .with_files()?
                                .build();
                            send_message_params.set_inline_keyboard_markup(keyboard);
                        }
                        Command::RenameFile => {
                            cs.set_action(ChatSessionAction::RenameFile(None));

                            send_message_params
                                .set_text(rename_file_message(cs.current_path_string()));

                            let keyboard = KeyboardDirectoryBuilder::new(&fs, cs.current_path())?
                                .with_files()?
                                .build();
                            send_message_params.set_inline_keyboard_markup(keyboard);
                        }
                        Command::MoveFile => {
                            cs.set_action(ChatSessionAction::MoveFile(None));

                            send_message_params
                                .set_text(move_file_select_file_message(cs.current_path_string()));

                            let keyboard = KeyboardDirectoryBuilder::new(&fs, cs.current_path())?
                                .with_files()?
                                .build();
                            send_message_params.set_inline_keyboard_markup(keyboard);
                        }
                        Command::DeleteDir | Command::DeleteFile => {
                            send_message_params.set_text(COMING_SOON_TEXT.to_string());
                        }
                    }

                    Ok(send_message_params)
                }
                Err(_) => {
                    if let Some(text) = msg.text {
                        return match cs.action() {
                            Some(current_action) => match current_action {
                                ChatSessionAction::MkDir(Some(
                                    ChatSessionWaitReply::DirectoryName,
                                )) => {
                                    let dir_name = text;
                                    let dir_path = cs.current_path().join(&dir_name);
                                    fs.mkdir(&dir_path)?;
                                    cs.reset();

                                    let mut send_message_params =
                                        MessageParams::new_send(chat_id.clone());
                                    send_message_params.set_text(
                                        created_directory_success_message(
                                            dir_name,
                                            dir_path.to_string_lossy().to_string(),
                                        ),
                                    );
                                    Ok(send_message_params)
                                }
                                ChatSessionAction::SaveFile(
                                    Some(file_node),
                                    Some(ChatSessionWaitReply::FileName),
                                ) => {
                                    let file_name = text;
                                    let dir_path = cs.current_path();
                                    let file_path = dir_path.join(file_name);
                                    let final_file_path =
                                        fs.create_file_from_node(&file_path, file_node)?;
                                    let mut send_message_params =
                                        MessageParams::new_send(chat_id.clone());
                                    send_message_params.set_text(created_file_success_message(
                                        final_file_path
                                            .file_name()
                                            .unwrap()
                                            .to_string_lossy()
                                            .to_string(),
                                        dir_path.to_string_lossy().to_string(),
                                    ));
                                    Ok(send_message_params)
                                }
                                ChatSessionAction::RenameFile(Some(
                                    ChatSessionWaitReply::FileName,
                                )) => {
                                    let new_file_name = text;
                                    let from_path = cs.current_path();
                                    let mut to_path = from_path.clone();
                                    to_path.set_file_name(&new_file_name);
                                    fs.mv(from_path, &to_path)?;
                                    let mut send_message_params =
                                        MessageParams::new_send(chat_id.clone());
                                    send_message_params.set_text(renamed_file_success_message(
                                        from_path
                                            .file_name()
                                            .unwrap()
                                            .to_string_lossy()
                                            .to_string(),
                                        new_file_name,
                                        from_path.parent().unwrap().to_string_lossy().to_string(),
                                    ));
                                    Ok(send_message_params)
                                }
                                _ => Ok(MessageParams::generic_error(chat_id.clone())),
                            },
                            None => process_file_message(
                                cs,
                                &fs,
                                chat_id.clone(),
                                msg.message_id,
                                Some(text.len().try_into().unwrap()),
                                Some(format!("{TG_FILE_MIME_TYPE_PREFIX}text")),
                            ),
                        };
                    };

                    if let Some(document) = msg.document {
                        return process_file_message(
                            cs,
                            &fs,
                            chat_id.clone(),
                            msg.message_id,
                            document.file_size,
                            document.mime_type,
                        );
                    }

                    if let Some(photos) = msg.photo {
                        let photo = photos.first().unwrap();
                        return process_file_message(
                            cs,
                            &fs,
                            chat_id.clone(),
                            msg.message_id,
                            photo.file_size,
                            Some("jpeg".to_string()),
                        );
                    }

                    if let Some(video) = msg.video {
                        return process_file_message(
                            cs,
                            &fs,
                            chat_id.clone(),
                            msg.message_id,
                            video.file_size,
                            video.mime_type,
                        );
                    }

                    if let Some(video_note) = msg.video_note {
                        return process_file_message(
                            cs,
                            &fs,
                            chat_id.clone(),
                            msg.message_id,
                            video_note.file_size,
                            Some(format!("{TG_FILE_MIME_TYPE_PREFIX}video_note")),
                        );
                    }

                    if let Some(audio) = msg.audio {
                        return process_file_message(
                            cs,
                            &fs,
                            chat_id.clone(),
                            msg.message_id,
                            audio.file_size,
                            audio.mime_type,
                        );
                    }

                    if let Some(voice) = msg.voice {
                        return process_file_message(
                            cs,
                            &fs,
                            chat_id.clone(),
                            msg.message_id,
                            voice.file_size,
                            voice.mime_type,
                        );
                    }

                    if let Some(sticker) = msg.sticker {
                        return process_file_message(
                            cs,
                            &fs,
                            chat_id.clone(),
                            msg.message_id,
                            sticker.file_size,
                            Some(format!("{TG_FILE_MIME_TYPE_PREFIX}sticker")),
                        );
                    }

                    if msg.contact.is_some() {
                        return process_file_message(
                            cs,
                            &fs,
                            chat_id.clone(),
                            msg.message_id,
                            None,
                            Some(format!("{TG_FILE_MIME_TYPE_PREFIX}contact")),
                        );
                    }

                    Ok(MessageParams::generic_error(chat_id.clone()))
                }
            }
        });

        self.save_chat_session_and_filesystem(chat_id, chat_session, fs);

        res
    }

    fn handle_update_content_callback_query(
        &self,
        chat_id: ChatId,
        query: CallbackQuery,
    ) -> Result<MessageParams, String> {
        let mut fs = self.filesystem_service.get_or_create_filesystem(&chat_id);
        let mut chat_session = self.get_or_create_chat_session(&chat_id);

        let res = with_clear_action_on_error(&mut chat_session, |cs| {
            let action = query
                .data
                .ok_or_else(|| "Data not found in callback query".to_string())?
                .into();
            let message_id = match query
                .message
                .ok_or_else(|| "Message not found in callback query".to_string())?
            {
                MaybeInaccessibleMessage::Message(msg) => msg.message_id,
                MaybeInaccessibleMessage::InaccessibleMessage(msg) => msg.message_id,
            };

            custom_print!(
                    "UpdateContent::CallbackQuery: chat_id: {:?}, current_path: {:?}, current_action: {:?}, action: {:?}",
                    chat_id,
                    cs.current_path(),
                    cs.action(),
                    action
                );

            let mut edit_message_params = MessageParams::new_edit(chat_id.clone(), message_id);
            let current_action = cs.action().ok_or_else(|| {
                "UpdateContent::CallbackQuery: No action in chat session".to_string()
            })?;

            match action {
                ChatSessionAction::CurrentDir => match current_action {
                    ChatSessionAction::MkDir(None) => {
                        cs.set_action(ChatSessionAction::MkDir(Some(
                            ChatSessionWaitReply::DirectoryName,
                        )));
                        edit_message_params
                            .set_text(ask_directory_name_message(cs.current_path_string()));
                        edit_message_params.set_inline_keyboard_markup(back_inline_keyboard());

                        Ok(edit_message_params)
                    }
                    ChatSessionAction::SaveFile(Some(file_node), None) => {
                        cs.set_action(ChatSessionAction::SaveFile(
                            Some(file_node),
                            Some(ChatSessionWaitReply::FileName),
                        ));
                        edit_message_params
                            .set_text(ask_file_name_message(cs.current_path_string()));
                        edit_message_params.set_inline_keyboard_markup(back_inline_keyboard());

                        Ok(edit_message_params)
                    }
                    ChatSessionAction::MoveFile(Some(from_path)) => {
                        let file_name =
                            from_path.file_name().unwrap().to_string_lossy().to_string();
                        let to_path = cs.current_path().join(&file_name);
                        fs.mv(&from_path, &to_path)?;

                        edit_message_params.set_text(moved_file_success_message(
                            file_name,
                            from_path.to_string_lossy().to_string(),
                            to_path.to_string_lossy().to_string(),
                        ));
                        Ok(edit_message_params)
                    }
                    _ => action_not_supported_error(),
                },
                ChatSessionAction::ParentDir => {
                    let current_path = cs.current_path().clone();
                    let root_path = root_path();
                    let parent_path = current_path.parent().unwrap_or(root_path.as_ref());

                    match current_action {
                        ChatSessionAction::Explorer => {
                            let node = fs.get_node(parent_path)?;

                            if node.is_directory() {
                                cs.set_current_path(parent_path.to_path_buf());
                                edit_message_params
                                    .set_text(explorer_message(cs.current_path_string()));

                                let keyboard = KeyboardDirectoryBuilder::new(&fs, parent_path)?
                                    .with_files()?
                                    .build();
                                edit_message_params.set_inline_keyboard_markup(keyboard);
                            } else {
                                // should never happen
                                return Err("Parent is not a directory".to_string());
                            }

                            Ok(edit_message_params)
                        }
                        ChatSessionAction::MkDir(_) => {
                            cs.set_current_path(parent_path.to_path_buf());
                            edit_message_params.set_text(mkdir_message(cs.current_path_string()));

                            let keyboard = KeyboardDirectoryBuilder::new(&fs, parent_path)?
                                .with_current_dir_button()
                                .build();
                            edit_message_params.set_inline_keyboard_markup(keyboard);
                            Ok(edit_message_params)
                        }
                        ChatSessionAction::SaveFile(Some(_), None) => {
                            cs.set_current_path(parent_path.to_path_buf());
                            edit_message_params
                                .set_text(create_file_message(cs.current_path_string()));

                            let keyboard = KeyboardDirectoryBuilder::new(&fs, parent_path)?
                                .with_current_dir_button()
                                .build();
                            edit_message_params.set_inline_keyboard_markup(keyboard);
                            Ok(edit_message_params)
                        }
                        ChatSessionAction::RenameFile(_) => {
                            cs.set_current_path(parent_path.to_path_buf());
                            edit_message_params
                                .set_text(rename_file_message(cs.current_path_string()));

                            let keyboard = KeyboardDirectoryBuilder::new(&fs, parent_path)?
                                .with_files()?
                                .build();
                            edit_message_params.set_inline_keyboard_markup(keyboard);
                            Ok(edit_message_params)
                        }
                        ChatSessionAction::MoveFile(from_path) => {
                            cs.set_current_path(parent_path.to_path_buf());

                            let (message_text, keyboard) = match from_path {
                                Some(from_path) => {
                                    let msg = move_file_select_destination_message(
                                        from_path.to_string_lossy().to_string(),
                                    );
                                    let keyboard =
                                        KeyboardDirectoryBuilder::new(&fs, cs.current_path())?
                                            .with_current_dir_button()
                                            .build();
                                    (msg, keyboard)
                                }
                                None => {
                                    let msg =
                                        move_file_select_file_message(cs.current_path_string());
                                    let keyboard = KeyboardDirectoryBuilder::new(&fs, parent_path)?
                                        .with_files()?
                                        .build();
                                    (msg, keyboard)
                                }
                            };
                            edit_message_params.set_text(message_text);
                            edit_message_params.set_inline_keyboard_markup(keyboard);

                            Ok(edit_message_params)
                        }
                        _ => action_not_supported_error(),
                    }
                }
                ChatSessionAction::FileOrDir(path) => match current_action {
                    ChatSessionAction::Explorer => {
                        let node = fs.get_node(&path)?;

                        if node.is_directory() {
                            cs.set_current_path(path.clone());
                            edit_message_params
                                .set_text(explorer_message(cs.current_path_string()));

                            let keyboard = KeyboardDirectoryBuilder::new(&fs, &path)?
                                .with_files()?
                                .build();
                            edit_message_params.set_inline_keyboard_markup(keyboard);
                        } else {
                            // reply to the file
                            let message_id = node
                                .file_message_id()
                                .ok_or_else(|| "Message id not found".to_string())?;
                            let file_name = path
                                .file_name()
                                .ok_or_else(|| "File name not found".to_string())?
                                .to_string_lossy()
                                .to_string();

                            let mut send_message_params = MessageParams::new_send(chat_id.clone());
                            send_message_params.set_text(explorer_file_message(
                                file_name,
                                cs.current_path_string(),
                            ));
                            send_message_params.set_reply_to_message_id(message_id)?;

                            return Ok(send_message_params);
                        }

                        Ok(edit_message_params)
                    }
                    ChatSessionAction::MkDir(_) => {
                        cs.set_current_path(path.clone());
                        edit_message_params.set_text(mkdir_message(cs.current_path_string()));

                        let keyboard = KeyboardDirectoryBuilder::new(&fs, &path)?
                            .with_current_dir_button()
                            .build();
                        edit_message_params.set_inline_keyboard_markup(keyboard);
                        Ok(edit_message_params)
                    }
                    ChatSessionAction::SaveFile(Some(_), None) => {
                        cs.set_current_path(path.clone());
                        edit_message_params.set_text(create_file_message(cs.current_path_string()));

                        let keyboard = KeyboardDirectoryBuilder::new(&fs, &path)?
                            .with_current_dir_button()
                            .build();
                        edit_message_params.set_inline_keyboard_markup(keyboard);
                        Ok(edit_message_params)
                    }
                    ChatSessionAction::RenameFile(None) => {
                        let node = fs.get_node(&path)?;

                        if node.is_directory() {
                            cs.set_current_path(path.clone());
                            edit_message_params
                                .set_text(rename_file_message(cs.current_path_string()));

                            let keyboard = KeyboardDirectoryBuilder::new(&fs, cs.current_path())?
                                .with_files()?
                                .build();
                            edit_message_params.set_inline_keyboard_markup(keyboard);
                        } else {
                            // reply to the file
                            let message_id = node
                                .file_message_id()
                                .ok_or_else(|| "Message id not found".to_string())?;
                            let file_name = path
                                .file_name()
                                .ok_or_else(|| "File name not found".to_string())?
                                .to_string_lossy()
                                .to_string();

                            let mut send_message_params = MessageParams::new_send(chat_id.clone());
                            send_message_params.set_text(ask_rename_file_message(
                                file_name,
                                cs.current_path_string(),
                            ));
                            send_message_params.set_reply_to_message_id(message_id)?;

                            cs.set_current_path(path);
                            cs.set_action(ChatSessionAction::RenameFile(Some(
                                ChatSessionWaitReply::FileName,
                            )));

                            return Ok(send_message_params);
                        }

                        Ok(edit_message_params)
                    }
                    ChatSessionAction::MoveFile(from_path) => {
                        let node = fs.get_node(&path)?;

                        if node.is_directory() {
                            cs.set_current_path(path.clone());

                            let (message_text, keyboard) = match from_path {
                                Some(from_path) => {
                                    let msg = move_file_select_destination_message(
                                        from_path.to_string_lossy().to_string(),
                                    );
                                    let keyboard =
                                        KeyboardDirectoryBuilder::new(&fs, cs.current_path())?
                                            .with_current_dir_button()
                                            .build();
                                    (msg, keyboard)
                                }
                                None => {
                                    let msg =
                                        move_file_select_file_message(cs.current_path_string());
                                    let keyboard =
                                        KeyboardDirectoryBuilder::new(&fs, cs.current_path())?
                                            .with_files()?
                                            .build();
                                    (msg, keyboard)
                                }
                            };
                            edit_message_params.set_text(message_text);
                            edit_message_params.set_inline_keyboard_markup(keyboard);
                        } else {
                            // reply to the file
                            let message_id = node
                                .file_message_id()
                                .ok_or_else(|| "Message id not found".to_string())?;
                            let from_path = path.clone();

                            cs.set_current_path(root_path());

                            let mut send_message_params = MessageParams::new_send(chat_id.clone());
                            send_message_params.set_text(move_file_select_destination_message(
                                from_path.to_string_lossy().to_string(),
                            ));
                            send_message_params.set_reply_to_message_id(message_id)?;
                            let keyboard = KeyboardDirectoryBuilder::new(&fs, cs.current_path())?
                                .with_current_dir_button()
                                .build();
                            send_message_params.set_inline_keyboard_markup(keyboard);

                            cs.set_action(ChatSessionAction::MoveFile(Some(from_path)));

                            return Ok(send_message_params);
                        }

                        Ok(edit_message_params)
                    }
                    _ => action_not_supported_error(),
                },
                ChatSessionAction::Back => match current_action {
                    ChatSessionAction::MkDir(Some(_)) => {
                        cs.set_action(ChatSessionAction::MkDir(None));

                        edit_message_params.set_text(mkdir_message(cs.current_path_string()));

                        let keyboard = KeyboardDirectoryBuilder::new(&fs, cs.current_path())?
                            .with_current_dir_button()
                            .build();
                        edit_message_params.set_inline_keyboard_markup(keyboard);

                        Ok(edit_message_params)
                    }
                    ChatSessionAction::SaveFile(Some(file_node), Some(_)) => {
                        cs.set_action(ChatSessionAction::SaveFile(Some(file_node), None));

                        edit_message_params.set_text(create_file_message(cs.current_path_string()));

                        let keyboard = KeyboardDirectoryBuilder::new(&fs, cs.current_path())?
                            .with_current_dir_button()
                            .build();
                        edit_message_params.set_inline_keyboard_markup(keyboard);

                        Ok(edit_message_params)
                    }
                    _ => action_not_supported_error(),
                },
                ChatSessionAction::DeleteDir
                | ChatSessionAction::Explorer
                | ChatSessionAction::MoveFile(_)
                | ChatSessionAction::DeleteFile
                | ChatSessionAction::SaveFile(_, _)
                | ChatSessionAction::RenameFile(_)
                | ChatSessionAction::MkDir(_) => Err("invalid action".to_string()),
            }
        });

        self.save_chat_session_and_filesystem(chat_id, chat_session, fs);

        res
    }
}

impl<T: ChatSessionRepository, F: FilesystemService> ChatSessionServiceImpl<T, F> {
    fn new(chat_session_repository: T, filesystem_service: F) -> Self {
        Self {
            chat_session_repository,
            filesystem_service,
        }
    }

    fn save_chat_session_and_filesystem(
        &self,
        chat_id: ChatId,
        chat_session: ChatSession,
        filesystem: FileSystem,
    ) {
        self.update_chat_session(chat_id.clone(), chat_session);
        self.filesystem_service
            .update_filesystem(&chat_id, filesystem);
    }
}

fn process_file_message(
    chat_session: &mut ChatSession,
    fs: &FileSystem,
    chat_id: ChatId,
    message_id: MessageId,
    file_size: Option<u64>,
    mime_type: Option<String>,
) -> Result<MessageParams, String> {
    // we reset the chat session to start the flow of saving a new file
    chat_session.reset();

    let file_node = FileSystemNode::new_file(message_id, file_size.unwrap_or(0), mime_type);
    chat_session.set_action(ChatSessionAction::SaveFile(Some(file_node), None));

    let mut send_message_params = MessageParams::new_send(chat_id.clone());
    send_message_params.set_text(create_file_message(chat_session.current_path_string()));
    let keyboard = KeyboardDirectoryBuilder::new(fs, chat_session.current_path())?
        .with_current_dir_button()
        .build();
    send_message_params.set_inline_keyboard_markup(keyboard);

    Ok(send_message_params)
}

fn action_not_supported_error() -> Result<MessageParams, String> {
    Err("current action not supported by this action".to_string())
}
