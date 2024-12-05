use frankenstein::{
    EditMessageTextParams, InlineKeyboardMarkup, LinkPreviewOptions, MaybeInaccessibleMessage,
    ParseMode, ReplyMarkup, ReplyParameters, SendMessageParams, Update, UpdateContent,
};
use ic_cdk::{query, update};
use serde_json::Value;

use crate::{
    custom_print,
    repositories::{
        with_clear_action_on_error, ChatId, ChatSession, ChatSessionAction,
        ChatSessionRepositoryImpl, ChatSessionWaitReply, Command, FileSystem, FileSystemNode,
        FilesystemRepositoryImpl, HeaderField, HttpRequest, HttpResponse, HttpUpdateRequest,
        KeyboardDirectoryBuilder, MessageId,
    },
    services::{
        ChatSessionService, ChatSessionServiceImpl, FilesystemService, FilesystemServiceImpl,
    },
    utils::{
        filesystem::root_path,
        http::{error500, ok200},
        messages::{
            ask_directory_name_message, ask_file_name_message, back_inline_keyboard,
            create_file_message, created_directory_success_message, created_file_success_message,
            delete_dir_message, delete_file_message, explorer_file_message, explorer_message,
            generic_error_message, help_message, info_message, mkdir_message,
            prepare_move_file_message, rename_file_message, start_message,
        },
        TG_FILE_MIME_TYPE_PREFIX,
    },
};

#[query]
fn http_request(_req: HttpRequest) -> HttpResponse {
    HttpResponse {
        status_code: 101,
        headers: vec![],
        body: "".into(),
        streaming_strategy: None,
        upgrade: Some(true),
    }
}

#[update]
fn http_request_update(req: HttpUpdateRequest) -> HttpResponse {
    HttpController::default().http_request(req)
}

struct HttpController<F: FilesystemService, C: ChatSessionService> {
    filesystem_service: F,
    chat_session_service: C,
}

impl Default
    for HttpController<
        FilesystemServiceImpl<FilesystemRepositoryImpl>,
        ChatSessionServiceImpl<ChatSessionRepositoryImpl>,
    >
{
    fn default() -> Self {
        Self::new(
            FilesystemServiceImpl::default(),
            ChatSessionServiceImpl::default(),
        )
    }
}

fn add_method(value: &mut Value, method: String) {
    if let Value::Object(m) = value {
        m.insert("method".to_string(), Value::String(method));
    }
}

fn default_link_preview_options() -> LinkPreviewOptions {
    LinkPreviewOptions {
        is_disabled: Some(true),
        url: None,
        prefer_small_media: None,
        prefer_large_media: None,
        show_above_text: None,
    }
}

fn default_send_message_params(chat_id: ChatId) -> SendMessageParams {
    #[allow(deprecated)]
    // MarkdownV2 does not work, we have to use the deprecated Markdown variant
    SendMessageParams::builder()
        .chat_id(chat_id.into_tg_chat_id())
        .parse_mode(ParseMode::Markdown)
        .link_preview_options(default_link_preview_options())
        .text("")
        .build()
}

fn default_edit_message_params(chat_id: ChatId, message_id: i32) -> EditMessageTextParams {
    #[allow(deprecated)]
    // MarkdownV2 does not work, we have to use the deprecated Markdown variant
    EditMessageTextParams::builder()
        .chat_id(chat_id.into_tg_chat_id())
        .message_id(message_id)
        .parse_mode(ParseMode::Markdown)
        .link_preview_options(default_link_preview_options())
        .text("")
        .build()
}

enum MessageParams {
    Send(SendMessageParams),
    Edit(EditMessageTextParams),
}

impl MessageParams {
    fn new_send(chat_id: ChatId) -> Self {
        let params = default_send_message_params(chat_id);
        MessageParams::Send(params)
    }

    fn new_edit(chat_id: ChatId, message_id: i32) -> Self {
        let params = default_edit_message_params(chat_id, message_id);
        MessageParams::Edit(params)
    }

    fn method(&self) -> String {
        match self {
            Self::Send(_) => "sendMessage".to_string(),
            Self::Edit(_) => "editMessageText".to_string(),
        }
    }

    fn json_value(&self) -> Result<Value, String> {
        let mut value = match self {
            Self::Send(params) => serde_json::to_value(params),
            Self::Edit(params) => serde_json::to_value(params),
        }
        .map_err(|err| err.to_string())?;

        add_method(&mut value, self.method());

        Ok(value)
    }

    fn set_text(&mut self, text: String) {
        match self {
            Self::Send(params) => params.text = text,
            Self::Edit(params) => params.text = text,
        }
    }

    fn set_inline_keyboard_markup(&mut self, keyboard: InlineKeyboardMarkup) {
        match self {
            Self::Send(params) => {
                params.reply_markup = Some(ReplyMarkup::InlineKeyboardMarkup(keyboard))
            }
            Self::Edit(params) => params.reply_markup = Some(keyboard),
        }
    }

    fn set_parse_mode(&mut self, parse_mode: Option<ParseMode>) {
        match self {
            Self::Send(params) => params.parse_mode = parse_mode,
            Self::Edit(params) => params.parse_mode = parse_mode,
        }
    }

    fn set_reply_to_message_id(&mut self, message_id: MessageId) -> Result<(), String> {
        match self {
            Self::Send(params) => {
                params.reply_parameters =
                    Some(ReplyParameters::builder().message_id(message_id).build());
                Ok(())
            }
            Self::Edit(_) => {
                Err("editMessageText does not support reply_to_message_id".to_string())
            }
        }
    }

    fn generic_error(chat_id: ChatId) -> Self {
        let mut params = Self::new_send(chat_id);
        params.set_text(generic_error_message());
        params.set_parse_mode(None);
        params
    }
}

fn http_response(message_params: &MessageParams) -> Result<HttpResponse, String> {
    let value = message_params.json_value()?;

    Ok(HttpResponse {
        status_code: 200,
        headers: vec![HeaderField(
            String::from("content-type"),
            String::from("application/json"),
        )],
        body: serde_json::to_vec(&value).map_err(|err| err.to_string())?,
        upgrade: Some(false),
        streaming_strategy: None,
    })
}

impl<F: FilesystemService, C: ChatSessionService> HttpController<F, C> {
    fn new(filesystem_service: F, chat_session_service: C) -> Self {
        Self {
            filesystem_service,
            chat_session_service,
        }
    }

    // TODO: write integration tests for testing bot updates.
    // See https://core.telegram.org/bots/webhooks#testing-your-bot-with-updates
    fn http_request(&self, req: HttpUpdateRequest) -> HttpResponse {
        custom_print!(
            "http_request: method: {:?}, url: {:?}, body length: {:?}",
            req.method,
            req.url,
            req.body.len()
        );

        let update: Update = match serde_json::from_slice(&req.body) {
            Ok(update) => update,
            Err(err) => return error500(Some(err)),
        };

        match self.process_tg_update_content(update.content) {
            Ok(params) => http_response(&params),
            Err((err, None)) => Err(err),
            Err((err, Some(chat_id))) => {
                let err_msg = format!("Error processing update content: {}", err);
                custom_print!("{}", err_msg);
                let mut params = default_send_message_params(chat_id);
                params.text = err_msg;
                params.parse_mode = None;
                http_response(&MessageParams::Send(params))
            }
        }
        .unwrap_or_else(|err| {
            custom_print!("Error: {}", err);
            // returning 500 causes Telegram to retry the request, which is not what we want in this case
            ok200()
        })
    }

    fn process_tg_update_content(
        &self,
        update_content: UpdateContent,
    ) -> Result<MessageParams, (String, Option<ChatId>)> {
        let chat_id = ChatId::try_from(&update_content).map_err(|err| (err, None))?;
        custom_print!("Message from chat_id: {}", chat_id);

        let mut fs = self.filesystem_service.get_or_create_filesystem(&chat_id);
        let mut chat_session = self
            .chat_session_service
            .get_or_create_chat_session(&chat_id);

        let res = match update_content {
            UpdateContent::Message(msg) => {
                let from_user = msg.clone().from;

                with_clear_action_on_error(&mut chat_session, |cs| {
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
                                    send_message_params.set_text(start_message(
                                        from_user.map(|user| user.first_name),
                                    ));
                                }
                                Command::Help => send_message_params.set_text(help_message()),
                                Command::Info => send_message_params.set_text(info_message()),
                                Command::MkDir => {
                                    cs.set_action(ChatSessionAction::MkDir(None));

                                    send_message_params
                                        .set_text(mkdir_message(cs.current_path_string()));

                                    let keyboard =
                                        KeyboardDirectoryBuilder::new(&fs, cs.current_path())?
                                            .with_current_dir_button()
                                            .build();
                                    send_message_params.set_inline_keyboard_markup(keyboard);
                                }
                                Command::Explorer => {
                                    cs.set_action(ChatSessionAction::Explorer);

                                    send_message_params
                                        .set_text(explorer_message(cs.current_path_string()));

                                    let keyboard =
                                        KeyboardDirectoryBuilder::new(&fs, cs.current_path())?
                                            .with_files()?
                                            .build();
                                    send_message_params.set_inline_keyboard_markup(keyboard);
                                }
                                Command::RenameFile => {
                                    cs.set_action(ChatSessionAction::RenameFile(None));

                                    send_message_params
                                        .set_text(rename_file_message(cs.current_path_string()));

                                    let keyboard =
                                        KeyboardDirectoryBuilder::new(&fs, cs.current_path())?
                                            .with_files()?
                                            .build();
                                    send_message_params.set_inline_keyboard_markup(keyboard);
                                }
                                Command::MoveFile => {
                                    cs.set_action(ChatSessionAction::PrepareMoveFile);

                                    send_message_params.set_text(prepare_move_file_message(
                                        cs.current_path_string(),
                                    ));

                                    let keyboard =
                                        KeyboardDirectoryBuilder::new(&fs, cs.current_path())?
                                            .with_files()?
                                            .build();
                                    send_message_params.set_inline_keyboard_markup(keyboard);
                                }
                                Command::DeleteDir => {
                                    cs.set_action(ChatSessionAction::DeleteDir);

                                    send_message_params
                                        .set_text(delete_dir_message(cs.current_path_string()));

                                    let keyboard =
                                        KeyboardDirectoryBuilder::new(&fs, cs.current_path())?
                                            .with_delete_dir_button()
                                            .with_files()?
                                            .build();
                                    send_message_params.set_inline_keyboard_markup(keyboard);
                                }
                                Command::DeleteFile => {
                                    cs.set_action(ChatSessionAction::DeleteFile);

                                    send_message_params
                                        .set_text(delete_file_message(cs.current_path_string()));

                                    let keyboard =
                                        KeyboardDirectoryBuilder::new(&fs, cs.current_path())?
                                            .with_files()?
                                            .build();
                                    send_message_params.set_inline_keyboard_markup(keyboard);
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
                                        ChatSessionAction::SaveFile(Some(file_node), Some(_)) => {
                                            let file_name = text;
                                            let dir_path = cs.current_path();
                                            let file_path = dir_path.join(file_name);
                                            let final_file_path =
                                                fs.create_file_from_node(&file_path, file_node)?;
                                            let mut send_message_params =
                                                MessageParams::new_send(chat_id.clone());
                                            send_message_params.set_text(
                                                created_file_success_message(
                                                    final_file_path
                                                        .file_name()
                                                        .unwrap()
                                                        .to_string_lossy()
                                                        .to_string(),
                                                    dir_path.to_string_lossy().to_string(),
                                                ),
                                            );
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
                })
            }
            UpdateContent::CallbackQuery(query) => {
                with_clear_action_on_error(&mut chat_session, |cs| {
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

                    let mut edit_message_params =
                        MessageParams::new_edit(chat_id.clone(), message_id);
                    let current_action = cs.action().ok_or_else(|| {
                        "UpdateContent::CallbackQuery: No action in chat session".to_string()
                    })?;

                    match action {
                        ChatSessionAction::CurrentDir => match current_action {
                            ChatSessionAction::MkDir(_) => {
                                cs.set_action(ChatSessionAction::MkDir(Some(
                                    ChatSessionWaitReply::DirectoryName,
                                )));
                                edit_message_params
                                    .set_text(ask_directory_name_message(cs.current_path_string()));
                                edit_message_params
                                    .set_inline_keyboard_markup(back_inline_keyboard());

                                Ok(edit_message_params)
                            }
                            ChatSessionAction::SaveFile(Some(file_node), None) => {
                                cs.set_action(ChatSessionAction::SaveFile(
                                    Some(file_node),
                                    Some(ChatSessionWaitReply::FileName),
                                ));
                                edit_message_params
                                    .set_text(ask_file_name_message(cs.current_path_string()));
                                edit_message_params
                                    .set_inline_keyboard_markup(back_inline_keyboard());

                                Ok(edit_message_params)
                            }
                            _ => Err("current action not supported by this action".to_string()),
                        },
                        ChatSessionAction::ParentDir => match current_action {
                            ChatSessionAction::Explorer => {
                                let current_path = cs.current_path().clone();
                                let root_path = root_path();
                                let path = current_path.parent().unwrap_or(root_path.as_ref());
                                let node = fs.get_node(path)?;

                                if node.is_directory() {
                                    cs.set_current_path(path.to_path_buf());
                                    edit_message_params
                                        .set_text(explorer_message(cs.current_path_string()));

                                    let keyboard = KeyboardDirectoryBuilder::new(&fs, path)?
                                        .with_files()?
                                        .build();
                                    edit_message_params.set_inline_keyboard_markup(keyboard);
                                }

                                Ok(edit_message_params)
                            }
                            ChatSessionAction::MkDir(_) => {
                                let current_path = cs.current_path().clone();
                                let root_path = root_path();
                                let path = current_path.parent().unwrap_or(root_path.as_ref());
                                cs.set_current_path(path.to_path_buf());
                                edit_message_params
                                    .set_text(mkdir_message(cs.current_path_string()));

                                let keyboard = KeyboardDirectoryBuilder::new(&fs, path)?
                                    .with_current_dir_button()
                                    .build();
                                edit_message_params.set_inline_keyboard_markup(keyboard);
                                Ok(edit_message_params)
                            }
                            _ => Err("current action not supported by this action".to_string()),
                        },
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

                                    let mut send_message_params =
                                        MessageParams::new_send(chat_id.clone());
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
                                edit_message_params
                                    .set_text(mkdir_message(cs.current_path_string()));

                                let keyboard = KeyboardDirectoryBuilder::new(&fs, &path)?
                                    .with_current_dir_button()
                                    .build();
                                edit_message_params.set_inline_keyboard_markup(keyboard);
                                Ok(edit_message_params)
                            }
                            ChatSessionAction::SaveFile(Some(_), None) => {
                                cs.set_current_path(path.clone());
                                edit_message_params
                                    .set_text(create_file_message(cs.current_path_string()));

                                let keyboard = KeyboardDirectoryBuilder::new(&fs, &path)?
                                    .with_current_dir_button()
                                    .build();
                                edit_message_params.set_inline_keyboard_markup(keyboard);
                                Ok(edit_message_params)
                            }
                            _ => Err("current action not supported by this action".to_string()),
                        },
                        ChatSessionAction::Back => match current_action {
                            ChatSessionAction::MkDir(Some(_)) => {
                                cs.set_action(ChatSessionAction::MkDir(None));

                                edit_message_params
                                    .set_text(mkdir_message(cs.current_path_string()));

                                let keyboard =
                                    KeyboardDirectoryBuilder::new(&fs, cs.current_path())?
                                        .with_current_dir_button()
                                        .build();
                                edit_message_params.set_inline_keyboard_markup(keyboard);

                                Ok(edit_message_params)
                            }
                            ChatSessionAction::SaveFile(Some(file_node), Some(_)) => {
                                cs.set_action(ChatSessionAction::SaveFile(Some(file_node), None));

                                edit_message_params
                                    .set_text(create_file_message(cs.current_path_string()));

                                let keyboard =
                                    KeyboardDirectoryBuilder::new(&fs, cs.current_path())?
                                        .with_current_dir_button()
                                        .build();
                                edit_message_params.set_inline_keyboard_markup(keyboard);

                                Ok(edit_message_params)
                            }
                            _ => Err("current action not supported by this action".to_string()),
                        },
                        ChatSessionAction::DeleteDir
                        | ChatSessionAction::Explorer
                        | ChatSessionAction::PrepareMoveFile
                        | ChatSessionAction::DeleteFile
                        | ChatSessionAction::SaveFile(_, _)
                        | ChatSessionAction::RenameFile(_)
                        | ChatSessionAction::MkDir(_) => Err("invalid action".to_string()),
                    }
                })
            }
            _ => Err("Unsupported update content".to_string()),
        };

        self.chat_session_service
            .update_chat_session(chat_id.clone(), chat_session);
        self.filesystem_service.update_filesystem(&chat_id, fs);

        res.map_err(|err| (err, Some(chat_id.clone())))
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
