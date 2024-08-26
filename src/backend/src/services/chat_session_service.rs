use crate::repositories::{ChatId, ChatSession, ChatSessionRepository, ChatSessionRepositoryImpl};

pub trait ChatSessionService {
    fn get_or_create_chat_session(&self, chat_id: &ChatId) -> ChatSession;

    fn update_chat_session(&self, chat_id: ChatId, chat_session: ChatSession);
}

pub struct ChatSessionServiceImpl<T: ChatSessionRepository> {
    chat_session_repository: T,
}

impl Default for ChatSessionServiceImpl<ChatSessionRepositoryImpl> {
    fn default() -> Self {
        Self::new(ChatSessionRepositoryImpl::default())
    }
}

impl<T: ChatSessionRepository> ChatSessionService for ChatSessionServiceImpl<T> {
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
}

impl<T: ChatSessionRepository> ChatSessionServiceImpl<T> {
    fn new(chat_session_repository: T) -> Self {
        Self {
            chat_session_repository,
        }
    }
}
