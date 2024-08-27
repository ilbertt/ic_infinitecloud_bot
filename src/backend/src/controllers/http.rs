use frankenstein::{
    LinkPreviewOptions, MaybeInaccessibleMessage, ParseMode, ReplyMarkup, SendMessageParams,
    Update, UpdateContent,
};
use ic_cdk::{query, update};
use serde_json::Value;

use crate::{
    repositories::{
        ChatId, ChatSessionAction, ChatSessionRepositoryImpl, Command, FilesystemRepositoryImpl,
        HeaderField, HttpRequest, HttpResponse, HttpUpdateRequest, KeyboardDirectoryBuilder,
    },
    services::{
        ChatSessionService, ChatSessionServiceImpl, FilesystemService, FilesystemServiceImpl,
    },
    utils::{
        http::{error500, ok200},
        messages::{
            delete_dir_message, delete_file_message, explorer_message, generic_error_message,
            help_message, info_message, mkdir_message, prepare_move_file_message,
            rename_file_message, start_message,
        },
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

fn default_send_message_params(chat_id: ChatId) -> SendMessageParams {
    #[allow(deprecated)]
    // MarkdownV2 does not work, we have to use the deprecated Markdown variant
    SendMessageParams::builder()
        .chat_id(chat_id.into_tg_chat_id())
        .parse_mode(ParseMode::Markdown)
        .link_preview_options(LinkPreviewOptions {
            is_disabled: Some(true),
            url: None,
            prefer_small_media: None,
            prefer_large_media: None,
            show_above_text: None,
        })
        .text("")
        .build()
}

fn send_message(msg: SendMessageParams) -> HttpResponse {
    let mut value = serde_json::to_value(msg).unwrap();
    add_method(&mut value, "sendMessage".to_string());

    HttpResponse {
        status_code: 200,
        headers: vec![HeaderField(
            String::from("content-type"),
            String::from("application/json"),
        )],
        body: serde_json::to_vec(&value).unwrap(),
        upgrade: Some(false),
        streaming_strategy: None,
    }
}

impl<F: FilesystemService, C: ChatSessionService> HttpController<F, C> {
    fn new(filesystem_service: F, chat_session_service: C) -> Self {
        Self {
            filesystem_service,
            chat_session_service,
        }
    }

    fn http_request(&self, req: HttpUpdateRequest) -> HttpResponse {
        let update: Update = match serde_json::from_slice(&req.body) {
            Ok(update) => update,
            Err(err) => return error500(Some(err)),
        };

        match update.content {
            UpdateContent::Message(msg) => {
                let chat_id = ChatId::from(msg.chat.id);
                let from_user = msg.clone().from;
                let fs = self.filesystem_service.get_or_create_filesystem(&chat_id);
                let mut chat_session = self
                    .chat_session_service
                    .get_or_create_chat_session(&chat_id);
                let current_path = chat_session.current_path().clone();

                let mut send_message_params = default_send_message_params(chat_id.clone());

                match Command::try_from(msg) {
                    Ok(command) => match command {
                        Command::Start => {
                            send_message_params.text =
                                start_message(from_user.map(|user| user.first_name))
                        }
                        Command::Help => send_message_params.text = help_message(),
                        Command::Info => send_message_params.text = info_message(),
                        Command::MkDir => {
                            chat_session.set_action(ChatSessionAction::MkDir);

                            send_message_params.text =
                                mkdir_message(chat_session.current_path_string());

                            let keyboard = KeyboardDirectoryBuilder::new(&fs, &current_path)
                                .with_current_dir_button()
                                .build();
                            send_message_params.reply_markup =
                                Some(ReplyMarkup::InlineKeyboardMarkup(keyboard));
                        }
                        Command::Explorer => {
                            chat_session.set_action(ChatSessionAction::Explorer);

                            send_message_params.text =
                                explorer_message(chat_session.current_path_string());

                            let keyboard = KeyboardDirectoryBuilder::new(&fs, &current_path)
                                .with_files()
                                .build();
                            send_message_params.reply_markup =
                                Some(ReplyMarkup::InlineKeyboardMarkup(keyboard));
                        }
                        Command::RenameFile => {
                            chat_session.set_action(ChatSessionAction::RenameFile);

                            send_message_params.text =
                                rename_file_message(chat_session.current_path_string());

                            let keyboard = KeyboardDirectoryBuilder::new(&fs, &current_path)
                                .with_files()
                                .build();
                            send_message_params.reply_markup =
                                Some(ReplyMarkup::InlineKeyboardMarkup(keyboard));
                        }
                        Command::MoveFile => {
                            chat_session.set_action(ChatSessionAction::PrepareMoveFile);

                            send_message_params.text =
                                prepare_move_file_message(chat_session.current_path_string());

                            let keyboard = KeyboardDirectoryBuilder::new(&fs, &current_path)
                                .with_files()
                                .build();
                            send_message_params.reply_markup =
                                Some(ReplyMarkup::InlineKeyboardMarkup(keyboard));
                        }
                        Command::DeleteDir => {
                            chat_session.set_action(ChatSessionAction::DeleteDir);

                            send_message_params.text =
                                delete_dir_message(chat_session.current_path_string());

                            let keyboard = KeyboardDirectoryBuilder::new(&fs, &current_path)
                                .with_delete_dir_button()
                                .with_files()
                                .build();
                            send_message_params.reply_markup =
                                Some(ReplyMarkup::InlineKeyboardMarkup(keyboard));
                        }
                        Command::DeleteFile => {
                            chat_session.set_action(ChatSessionAction::DeleteFile);

                            send_message_params.text =
                                delete_file_message(chat_session.current_path_string());

                            let keyboard = KeyboardDirectoryBuilder::new(&fs, &current_path)
                                .with_files()
                                .build();
                            send_message_params.reply_markup =
                                Some(ReplyMarkup::InlineKeyboardMarkup(keyboard));
                        }
                    },
                    Err(_) => send_message_params.text = "Error".to_string(),
                };

                self.chat_session_service
                    .update_chat_session(chat_id, chat_session);

                send_message(send_message_params)
            }
            UpdateContent::CallbackQuery(query) => {
                let chat_id = ChatId::from(
                    match query.message.unwrap() {
                        MaybeInaccessibleMessage::Message(msg) => msg.chat,
                        MaybeInaccessibleMessage::InaccessibleMessage(msg) => Box::new(msg.chat),
                    }
                    .id,
                );
                let from_user = query.from;
                let fs = self.filesystem_service.get_or_create_filesystem(&chat_id);
                let mut chat_session = self
                    .chat_session_service
                    .get_or_create_chat_session(&chat_id);

                let mut send_message_params = default_send_message_params(chat_id.clone());

                match query.data.unwrap().into() {
                    ChatSessionAction::MkDir => {}
                    ChatSessionAction::CurrentDir => {
                        let action = chat_session.action();

                        match action {
                            Some(ChatSessionAction::MkDir) => {}
                            _ => {
                                send_message_params.text = generic_error_message();
                                send_message_params.parse_mode = None;
                            }
                        }
                    }
                    ChatSessionAction::ParentDir => {}
                    ChatSessionAction::DeleteDir => {}
                    ChatSessionAction::Explorer => {}
                    ChatSessionAction::RenameFile => {}
                    ChatSessionAction::PrepareMoveFile => {}
                    ChatSessionAction::DeleteFile => {}
                }

                send_message(send_message_params)
            }
            _ => {
                // TODO: handle the other update types better

                ok200()
            }
        }
    }
}
