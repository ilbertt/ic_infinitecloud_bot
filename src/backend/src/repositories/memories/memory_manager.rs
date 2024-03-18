use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::DefaultMemoryImpl;
use std::cell::RefCell;

pub(super) type Memory = VirtualMemory<DefaultMemoryImpl>;

thread_local! {
    pub static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
}

// memory IDs are kept together to ensure that the same ID is not used more than once
// everything else related to each memory region is kept in the appropriate file
pub(super) const FILESYSTEM_MEMORY_ID: MemoryId = MemoryId::new(0);
pub(super) const CHAT_SESSION_MEMORY_ID: MemoryId = MemoryId::new(1);
