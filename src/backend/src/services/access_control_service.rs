use candid::Principal;
use ic_cdk::{api::is_controller, trap};

use crate::repositories::HttpUpdateRequest;

const TELEGRAM_WEBHOOK_SECRET_TOKEN_HEADER: &str = "x-telegram-bot-api-secret-token";
const TELEGRAM_WEBHOOK_SECRET_TOKEN: &str = env!("TELEGRAM_SECRET_TOKEN");

pub trait AccessControlService {
    fn assert_caller_is_controller(&self, calling_principal: &Principal);

    fn assert_http_request_is_authorized(&self, req: &HttpUpdateRequest) -> bool;
}

pub struct AccessControlServiceImpl {}

impl Default for AccessControlServiceImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl AccessControlService for AccessControlServiceImpl {
    fn assert_caller_is_controller(&self, calling_principal: &Principal) {
        if !is_controller(calling_principal) {
            trap("caller is not a controller");
        }
    }

    fn assert_http_request_is_authorized(&self, req: &HttpUpdateRequest) -> bool {
        req.headers.iter().any(|header| {
            header.0.to_lowercase() == TELEGRAM_WEBHOOK_SECRET_TOKEN_HEADER
                && header.1 == TELEGRAM_WEBHOOK_SECRET_TOKEN
        })
    }
}

impl AccessControlServiceImpl {
    fn new() -> Self {
        Self {}
    }
}
