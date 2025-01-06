use candid::Principal;
use ic_cdk::{caller, query};

use crate::{
    repositories::{ChatSessionRepositoryImpl, FilesystemRepositoryImpl},
    services::{
        AccessControlService, AccessControlServiceImpl, ChatSessionService, ChatSessionServiceImpl,
        FilesystemServiceImpl,
    },
};

#[query]
fn get_chat_sessions_count() -> u32 {
    let calling_principal = caller();

    ChatSessionController::default().get_chat_sessions_count(calling_principal)
}

struct ChatSessionController<A: AccessControlService, C: ChatSessionService> {
    access_control_service: A,
    chat_session_service: C,
}

impl Default
    for ChatSessionController<
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

impl<A: AccessControlService, C: ChatSessionService> ChatSessionController<A, C> {
    fn new(access_control_service: A, chat_session_service: C) -> Self {
        Self {
            access_control_service,
            chat_session_service,
        }
    }

    fn get_chat_sessions_count(&self, calling_principal: Principal) -> u32 {
        self.access_control_service
            .assert_caller_is_controller(&calling_principal);

        self.chat_session_service.get_chat_sessions_count()
    }
}
