use frankenstein::{
    LinkPreviewOptions, ParseMode, ReplyMarkup, SendMessageParams, Update, UpdateContent,
};
use ic_cdk::{query, update};
use serde_json::Value;

use crate::{
    custom_print,
    repositories::{
        with_clear_action_on_error, ChatId, ChatSessionAction, ChatSessionRepositoryImpl, Command,
        FilesystemRepositoryImpl, HeaderField, HttpRequest, HttpResponse, HttpUpdateRequest,
        KeyboardDirectoryBuilder,
    },
    services::{
        ChatSessionService, ChatSessionServiceImpl, FilesystemService, FilesystemServiceImpl,
    },
    utils::{
        filesystem::root_path,
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

fn send_message(msg: SendMessageParams) -> Result<HttpResponse, String> {
    let mut value = serde_json::to_value(msg).map_err(|err| err.to_string())?;
    add_method(&mut value, "sendMessage".to_string());

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
            Ok(params) => send_message(params),
            Err((err, None)) => Err(err),
            Err((err, Some(chat_id))) => {
                let err_msg = format!("Error processing update content: {}", err);
                custom_print!("{}", err_msg);
                let mut params = default_send_message_params(chat_id);
                params.text = err_msg;
                params.parse_mode = None;
                send_message(params)
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
    ) -> Result<SendMessageParams, (String, Option<ChatId>)> {
        let chat_id = ChatId::try_from(&update_content).map_err(|err| (err, None))?;
        custom_print!("Message from chat_id: {}", chat_id);
        match update_content {
            UpdateContent::Message(msg) => {
                let from_user = msg.clone().from;
                let fs = self.filesystem_service.get_or_create_filesystem(&chat_id);
                let mut chat_session = self
                    .chat_session_service
                    .get_or_create_chat_session(&chat_id);

                let res = with_clear_action_on_error(&mut chat_session, |cs| {
                    let current_path = cs.current_path().clone();
                    let command = Command::try_from(msg)?;

                    custom_print!(
                        "UpdateContent::Message: chat_id: {:?}, current_path: {:?}, current_action: {:?}, command: {:?}",
                        chat_id, current_path, cs.action(), command
                    );

                    let mut send_message_params = default_send_message_params(chat_id.clone());

                    match command {
                        Command::Start => {
                            send_message_params.text =
                                start_message(from_user.map(|user| user.first_name))
                        }
                        Command::Help => send_message_params.text = help_message(),
                        Command::Info => send_message_params.text = info_message(),
                        Command::MkDir => {
                            cs.set_action(ChatSessionAction::MkDir);

                            send_message_params.text = mkdir_message(cs.current_path_string());

                            let keyboard = KeyboardDirectoryBuilder::new(&fs, &current_path)?
                                .with_current_dir_button()
                                .build();
                            send_message_params.reply_markup =
                                Some(ReplyMarkup::InlineKeyboardMarkup(keyboard));
                        }
                        Command::Explorer => {
                            cs.set_action(ChatSessionAction::Explorer);

                            send_message_params.text = explorer_message(cs.current_path_string());

                            let keyboard = KeyboardDirectoryBuilder::new(&fs, &current_path)?
                                .with_files()?
                                .build();
                            send_message_params.reply_markup =
                                Some(ReplyMarkup::InlineKeyboardMarkup(keyboard));
                        }
                        Command::RenameFile => {
                            cs.set_action(ChatSessionAction::RenameFile);

                            send_message_params.text =
                                rename_file_message(cs.current_path_string());

                            let keyboard = KeyboardDirectoryBuilder::new(&fs, &current_path)?
                                .with_files()?
                                .build();
                            send_message_params.reply_markup =
                                Some(ReplyMarkup::InlineKeyboardMarkup(keyboard));
                        }
                        Command::MoveFile => {
                            cs.set_action(ChatSessionAction::PrepareMoveFile);

                            send_message_params.text =
                                prepare_move_file_message(cs.current_path_string());

                            let keyboard = KeyboardDirectoryBuilder::new(&fs, &current_path)?
                                .with_files()?
                                .build();
                            send_message_params.reply_markup =
                                Some(ReplyMarkup::InlineKeyboardMarkup(keyboard));
                        }
                        Command::DeleteDir => {
                            cs.set_action(ChatSessionAction::DeleteDir);

                            send_message_params.text = delete_dir_message(cs.current_path_string());

                            let keyboard = KeyboardDirectoryBuilder::new(&fs, &current_path)?
                                .with_delete_dir_button()
                                .with_files()?
                                .build();
                            send_message_params.reply_markup =
                                Some(ReplyMarkup::InlineKeyboardMarkup(keyboard));
                        }
                        Command::DeleteFile => {
                            cs.set_action(ChatSessionAction::DeleteFile);

                            send_message_params.text =
                                delete_file_message(cs.current_path_string());

                            let keyboard = KeyboardDirectoryBuilder::new(&fs, &current_path)?
                                .with_files()?
                                .build();
                            send_message_params.reply_markup =
                                Some(ReplyMarkup::InlineKeyboardMarkup(keyboard));
                        }
                    }

                    Ok(send_message_params)
                });

                self.chat_session_service
                        .update_chat_session(chat_id.clone(), chat_session);

                res
            }
            UpdateContent::CallbackQuery(query) => {
                let from_user = query.from;
                let fs = self.filesystem_service.get_or_create_filesystem(&chat_id);
                let mut chat_session = self
                    .chat_session_service
                    .get_or_create_chat_session(&chat_id);

                let res = with_clear_action_on_error(&mut chat_session, |cs| {
                    let action = query
                        .data
                        .ok_or_else(|| "Data not found in callback query".to_string())?
                        .into();

                    custom_print!(
                        "UpdateContent::CallbackQuery: chat_id: {:?}, current_path: {:?}, current_action: {:?}, action: {:?}",
                        chat_id,
                        cs.current_path(),
                        cs.action(),
                        action
                    );

                    let mut send_message_params = default_send_message_params(chat_id.clone());

                    match action {
                        ChatSessionAction::MkDir => {}
                        ChatSessionAction::CurrentDir => {
                            let current_action = cs.action();

                            match current_action {
                                Some(ChatSessionAction::MkDir) => {}
                                _ => {
                                    send_message_params.text = generic_error_message();
                                    send_message_params.parse_mode = None;
                                }
                            }
                        }
                        ChatSessionAction::ParentDir => match cs.action() {
                            Some(ChatSessionAction::Explorer) => {
                                let current_path = cs.current_path().clone();
                                let root_path = root_path();
                                let path = current_path.parent().unwrap_or(root_path.as_ref());
                                let node = fs.get_node(path)?;

                                if node.is_directory() {
                                    cs.set_current_path(path.to_path_buf());
                                    send_message_params.text =
                                        explorer_message(cs.current_path_string());

                                    let keyboard =
                                        KeyboardDirectoryBuilder::new(&fs, path)?
                                            .with_files()?
                                            .build();
                                    send_message_params.reply_markup =
                                        Some(ReplyMarkup::InlineKeyboardMarkup(keyboard));
                                }
                            }
                            _ => {
                                send_message_params.text = generic_error_message();
                                send_message_params.parse_mode = None;
                            }
                        },
                        ChatSessionAction::DeleteDir => {}
                        ChatSessionAction::Explorer => {}
                        ChatSessionAction::RenameFile => {}
                        ChatSessionAction::PrepareMoveFile => {}
                        ChatSessionAction::DeleteFile => {}
                        ChatSessionAction::FileOrDir(path) => match cs.action() {
                            Some(ChatSessionAction::Explorer) => {
                                let node = fs.get_node(&path)?;

                                if node.is_directory() {
                                    cs.set_current_path(path.clone());
                                    send_message_params.text =
                                        explorer_message(cs.current_path_string());

                                    let keyboard =
                                        KeyboardDirectoryBuilder::new(&fs, &path)?
                                            .with_files()?
                                            .build();
                                    send_message_params.reply_markup =
                                        Some(ReplyMarkup::InlineKeyboardMarkup(keyboard));
                                }
                            }
                            _ => {
                                send_message_params.text = generic_error_message();
                                send_message_params.parse_mode = None;
                            }
                        },
                    }

                    Ok(send_message_params)
                });

                self.chat_session_service
                        .update_chat_session(chat_id.clone(), chat_session);

                res
            }
            _ => Err("Unsupported update content".to_string()),
        }.map_err(|err| (err, Some(chat_id)))
    }
}
