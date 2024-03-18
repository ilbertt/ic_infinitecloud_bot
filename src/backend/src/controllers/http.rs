use frankenstein::{ParseMode, SendMessageParams, Update, UpdateContent};
use ic_cdk::{print, query, update};
use serde_json::Value;

use crate::{
    repositories::{
        ChatId, ChatSessionRepositoryImpl, Command, FilesystemRepositoryImpl, HeaderField,
        HttpRequest, HttpResponse, HttpUpdateRequest,
    },
    services::{
        ChatSessionService, ChatSessionServiceImpl, FilesystemService, FilesystemServiceImpl,
    },
    utils::{
        http::{error500, ok200},
        messages::{help_message, info_message, start_message},
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
    match value {
        Value::Object(m) => {
            m.insert("method".to_string(), Value::String(method));
        }
        _ => (),
    }
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
        match serde_json::from_slice::<Update>(&req.body) {
            Err(err) => error500(Some(err)),
            Ok(update) => {
                match update.content {
                    UpdateContent::Message(msg) => {
                        print(format!("Got message: {:?}", msg));

                        let chat_id = ChatId::from(msg.chat.id);
                        let from_user = msg.clone().from;
                        let fs = self.filesystem_service.get_or_create_filesystem(&chat_id);
                        let chat_session = self
                            .chat_session_service
                            .get_or_create_chat_session(&chat_id);

                        #[allow(deprecated)]
                        // MarkdownV2 does not work, we have to use the deprecated Markdown variant
                        let mut send_message_params = SendMessageParams::builder()
                            .chat_id(chat_id.into_tg_chat_id())
                            .parse_mode(ParseMode::Markdown)
                            .text("")
                            .build();

                        match Command::try_from(msg) {
                            Ok(command) => match command {
                                Command::Start => {
                                    send_message_params.text =
                                        start_message(from_user.map(|user| user.first_name))
                                }
                                Command::Help => send_message_params.text = help_message(),
                                Command::Info => send_message_params.text = info_message(),
                                Command::MkDir => {}
                            },
                            Err(_) => send_message_params.text = "Error".to_string(),
                        };

                        send_message(send_message_params)
                    }
                    _ => ok200(),
                }
            }
        }
    }
}
