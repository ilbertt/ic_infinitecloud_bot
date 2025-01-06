use candid::Principal;
use ic_cdk::{caller, query};

use crate::{
    repositories::{ChatSessionRepositoryImpl, FilesystemRepositoryImpl},
    services::{
        AdminService, AdminServiceImpl, ChatSessionService, ChatSessionServiceImpl,
        FilesystemServiceImpl,
    },
};

#[query]
fn get_chat_sessions_count() -> u32 {
    let calling_principal = caller();

    ChatSessionController::default().get_chat_sessions_count(calling_principal)
}

struct ChatSessionController<A: AdminService, C: ChatSessionService> {
    admin_service: A,
    chat_session_service: C,
}

impl Default
    for ChatSessionController<
        AdminServiceImpl,
        ChatSessionServiceImpl<
            ChatSessionRepositoryImpl,
            FilesystemServiceImpl<FilesystemRepositoryImpl>,
        >,
    >
{
    fn default() -> Self {
        Self::new(
            AdminServiceImpl::default(),
            ChatSessionServiceImpl::default(),
        )
    }
}

impl<A: AdminService, C: ChatSessionService> ChatSessionController<A, C> {
    fn new(admin_service: A, chat_session_service: C) -> Self {
        Self {
            admin_service,
            chat_session_service,
        }
    }

    fn get_chat_sessions_count(&self, calling_principal: Principal) -> u32 {
        self.admin_service
            .asset_caller_is_controller(&calling_principal);

        self.chat_session_service.get_chat_sessions_count()
    }
}
