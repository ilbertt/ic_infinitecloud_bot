use std::{borrow::Cow, fmt::Display};

use candid::{CandidType, Deserialize};
use frankenstein::{ChatId as TgChatId, MaybeInaccessibleMessage, UpdateContent};
use ic_stable_structures::{storable::Bound, Storable};

#[derive(Debug, CandidType, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChatId(pub u64);

impl From<i64> for ChatId {
    fn from(value: i64) -> Self {
        Self(value as u64)
    }
}

impl From<ChatId> for TgChatId {
    fn from(val: ChatId) -> Self {
        TgChatId::Integer(val.0 as i64)
    }
}

impl Display for ChatId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ChatId {
    pub fn into_tg_chat_id(self) -> TgChatId {
        self.into()
    }
}

impl Storable for ChatId {
    fn to_bytes(&self) -> Cow<[u8]> {
        self.0.to_bytes()
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Self(u64::from_bytes(bytes))
    }

    const BOUND: Bound = u64::BOUND;
}

impl TryFrom<&UpdateContent> for ChatId {
    type Error = String;

    fn try_from(update_content: &UpdateContent) -> Result<Self, Self::Error> {
        match update_content {
            UpdateContent::Message(msg) => Ok(ChatId::from(msg.chat.id)),
            UpdateContent::CallbackQuery(query) => Ok(ChatId::from(
                match query
                    .message
                    .as_ref()
                    .ok_or_else(|| "Message not found in callback query".to_string())?
                {
                    MaybeInaccessibleMessage::Message(msg) => msg.chat.clone(),
                    MaybeInaccessibleMessage::InaccessibleMessage(msg) => {
                        Box::new(msg.chat.clone())
                    }
                }
                .id,
            )),
            _ => Err("Unsupported update content".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[rstest]
    fn chat_id_storable_impl() {
        let chat_id = ChatId(123);

        let serialized_chat_id = chat_id.to_bytes();
        let deserialized_chat_id = ChatId::from_bytes(serialized_chat_id);

        assert_eq!(deserialized_chat_id, chat_id);
    }

    #[rstest]
    fn chat_id_from() {
        let from: ChatId = 123i64.into();

        assert_eq!(from, ChatId(123));
    }

    #[rstest]
    fn into_tg_chat_id() {
        let chat_id = ChatId(123);
        let tg_chat_id: TgChatId = chat_id.into_tg_chat_id();

        assert_eq!(tg_chat_id, TgChatId::Integer(123));
    }
}
