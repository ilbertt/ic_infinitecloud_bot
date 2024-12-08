use frankenstein::{
    EditMessageTextParams, InlineKeyboardMarkup, LinkPreviewOptions, ParseMode, ReplyMarkup,
    ReplyParameters, SendMessageParams,
};
use serde_json::Value;

use crate::repositories::{ChatId, MessageId};

use super::defaults::messages::generic_error_message;

fn add_method(value: &mut Value, method: String) {
    if let Value::Object(m) = value {
        m.insert("method".to_string(), Value::String(method));
    }
}

pub enum MessageParams {
    Send(SendMessageParams),
    Edit(EditMessageTextParams),
}

impl MessageParams {
    pub fn new_send(chat_id: ChatId) -> Self {
        let params = default_send_message_params(chat_id);
        MessageParams::Send(params)
    }

    pub fn new_edit(chat_id: ChatId, message_id: i32) -> Self {
        let params = default_edit_message_params(chat_id, message_id);
        MessageParams::Edit(params)
    }

    fn method(&self) -> String {
        match self {
            Self::Send(_) => "sendMessage".to_string(),
            Self::Edit(_) => "editMessageText".to_string(),
        }
    }

    pub fn json_value(&self) -> Result<Value, String> {
        let mut value = match self {
            Self::Send(params) => serde_json::to_value(params),
            Self::Edit(params) => serde_json::to_value(params),
        }
        .map_err(|err| err.to_string())?;

        add_method(&mut value, self.method());

        Ok(value)
    }

    pub fn set_text(&mut self, text: String) {
        match self {
            Self::Send(params) => params.text = text,
            Self::Edit(params) => params.text = text,
        }
    }

    pub fn set_inline_keyboard_markup(&mut self, keyboard: InlineKeyboardMarkup) {
        match self {
            Self::Send(params) => {
                params.reply_markup = Some(ReplyMarkup::InlineKeyboardMarkup(keyboard))
            }
            Self::Edit(params) => params.reply_markup = Some(keyboard),
        }
    }

    pub fn set_parse_mode(&mut self, parse_mode: Option<ParseMode>) {
        match self {
            Self::Send(params) => params.parse_mode = parse_mode,
            Self::Edit(params) => params.parse_mode = parse_mode,
        }
    }

    pub fn set_reply_to_message_id(&mut self, message_id: MessageId) -> Result<(), String> {
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

    pub fn generic_error(chat_id: ChatId) -> Self {
        let mut params = Self::new_send(chat_id);
        params.set_text(generic_error_message());
        params.set_parse_mode(None);
        params
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
