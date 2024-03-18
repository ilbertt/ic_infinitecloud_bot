use super::{Memory, CHAT_SESSION_MEMORY_ID, MEMORY_MANAGER};
use crate::repositories::{ChatId, ChatSession};
use ic_stable_structures::BTreeMap;

pub type ChatSessionMemory = BTreeMap<ChatId, ChatSession, Memory>;

pub fn init_chat_session() -> ChatSessionMemory {
    ChatSessionMemory::init(get_chat_session_memory())
}

fn get_chat_session_memory() -> Memory {
    MEMORY_MANAGER.with(|m| m.borrow().get(CHAT_SESSION_MEMORY_ID))
}
