use std::cell::RefCell;

use super::{init_chat_session, ChatId, ChatSession, ChatSessionMemory};

pub trait ChatSessionRepository {
    fn get_chat_session_by_chat_id(&self, chat_id: &ChatId) -> Option<ChatSession>;

    fn set_chat_session_by_chat_id(&self, chat_id: ChatId, chat_session: ChatSession);

    fn get_chat_session_count(&self) -> u64;
}

pub struct ChatSessionRepositoryImpl {}

impl Default for ChatSessionRepositoryImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl ChatSessionRepository for ChatSessionRepositoryImpl {
    fn get_chat_session_by_chat_id(&self, chat_id: &ChatId) -> Option<ChatSession> {
        STATE.with_borrow(|s| s.chat_session.get(chat_id))
    }

    fn set_chat_session_by_chat_id(&self, chat_id: ChatId, chat_session: ChatSession) {
        STATE.with_borrow_mut(|s| s.chat_session.insert(chat_id, chat_session));
    }

    fn get_chat_session_count(&self) -> u64 {
        STATE.with_borrow(|s| s.chat_session.len())
    }
}

impl ChatSessionRepositoryImpl {
    pub fn new() -> Self {
        Self {}
    }
}

struct SessionState {
    chat_session: ChatSessionMemory,
}

impl Default for SessionState {
    fn default() -> Self {
        Self {
            chat_session: init_chat_session(),
        }
    }
}

thread_local! {
    static STATE: RefCell<SessionState> = RefCell::new(SessionState::default());
}
