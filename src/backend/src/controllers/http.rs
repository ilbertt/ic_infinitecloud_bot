use frankenstein::{Update, UpdateContent};
use ic_cdk::{query, update};

use crate::{
    custom_print,
    repositories::{
        ChatId, ChatSessionRepositoryImpl, FilesystemRepositoryImpl, HeaderField, HttpRequest,
        HttpResponse, HttpUpdateRequest,
    },
    services::{
        AccessControlService, AccessControlServiceImpl, ChatSessionService, ChatSessionServiceImpl,
        FilesystemServiceImpl,
    },
    utils::{
        http::{error500, ok200},
        MessageParams,
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

struct HttpController<A: AccessControlService, C: ChatSessionService> {
    access_control_service: A,
    chat_session_service: C,
}

impl Default
    for HttpController<
        AccessControlServiceImpl,
        ChatSessionServiceImpl<
            ChatSessionRepositoryImpl,
            FilesystemServiceImpl<FilesystemRepositoryImpl>,
        >,
    >
{
    fn default() -> Self {
        Self::new(
            AccessControlServiceImpl::default(),
            ChatSessionServiceImpl::default(),
        )
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

impl<A: AccessControlService, C: ChatSessionService> HttpController<A, C> {
    fn new(access_control_service: A, chat_session_service: C) -> Self {
        Self {
            access_control_service,
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

        if !self
            .access_control_service
            .assert_http_request_is_authorized(&req)
        {
            custom_print!("Unauthorized request");
            // returning 40x causes Telegram to retry the request, which is not what we want
            return ok200();
        }

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
                let mut params = MessageParams::new_send(chat_id);
                params.set_text(err_msg);
                params.set_parse_mode(None);
                http_response(&params)
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

        let res = match update_content {
            UpdateContent::Message(msg) => self
                .chat_session_service
                .handle_update_content_message(chat_id.clone(), msg),
            UpdateContent::CallbackQuery(query) => self
                .chat_session_service
                .handle_update_content_callback_query(chat_id.clone(), query),
            _ => Err("Unsupported update content".to_string()),
        };

        res.map_err(|err| (err, Some(chat_id.clone())))
    }
}
