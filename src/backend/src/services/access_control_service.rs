use candid::Principal;
use ic_cdk::{api::is_controller, trap};

pub trait AccessControlService {
    fn asset_caller_is_controller(&self, calling_principal: &Principal);
}

pub struct AccessControlServiceImpl {}

impl Default for AccessControlServiceImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl AccessControlService for AccessControlServiceImpl {
    fn asset_caller_is_controller(&self, calling_principal: &Principal) {
        if !is_controller(calling_principal) {
            trap("caller is not a controller");
        }
    }
}

impl AccessControlServiceImpl {
    fn new() -> Self {
        Self {}
    }
}
