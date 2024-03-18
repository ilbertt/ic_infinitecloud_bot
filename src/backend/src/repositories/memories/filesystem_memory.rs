use super::{Memory, FILESYSTEM_MEMORY_ID, MEMORY_MANAGER};
use crate::repositories::{ChatId, Filesystem};
use ic_stable_structures::BTreeMap;

pub type FilesystemMemory = BTreeMap<ChatId, Filesystem, Memory>;

pub fn init_filesystem() -> FilesystemMemory {
    FilesystemMemory::init(get_filesystem_memory())
}

fn get_filesystem_memory() -> Memory {
    MEMORY_MANAGER.with(|m| m.borrow().get(FILESYSTEM_MEMORY_ID))
}
