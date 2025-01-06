use candid::Principal;
use ic_cdk::{api::is_controller, trap};

pub trait AdminService {
    fn asset_caller_is_controller(&self, calling_principal: &Principal);
}

pub struct AdminServiceImpl {}

impl Default for AdminServiceImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl AdminService for AdminServiceImpl {
    fn asset_caller_is_controller(&self, calling_principal: &Principal) {
        if !is_controller(calling_principal) {
            trap("caller is not a controller");
        }
    }
}

impl AdminServiceImpl {
    fn new() -> Self {
        Self {}
    }
}
