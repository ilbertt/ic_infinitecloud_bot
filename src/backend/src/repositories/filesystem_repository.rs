use std::cell::RefCell;

use super::{init_filesystem, ChatId, FileSystem, FilesystemMemory};

pub trait FilesystemRepository {
    fn get_filesystem_by_chat_id(&self, chat_id: &ChatId) -> Option<FileSystem>;

    fn set_filesystem_by_chat_id(&self, chat_id: ChatId, filesystem: FileSystem);
}

pub struct FilesystemRepositoryImpl {}

impl Default for FilesystemRepositoryImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl FilesystemRepository for FilesystemRepositoryImpl {
    fn get_filesystem_by_chat_id(&self, chat_id: &ChatId) -> Option<FileSystem> {
        STATE.with_borrow(|s| s.filesystem.get(chat_id))
    }

    fn set_filesystem_by_chat_id(&self, chat_id: ChatId, filesystem: FileSystem) {
        STATE.with_borrow_mut(|s| s.filesystem.insert(chat_id, filesystem));
    }
}

impl FilesystemRepositoryImpl {
    pub fn new() -> Self {
        Self {}
    }
}

struct FilesystemState {
    filesystem: FilesystemMemory,
}

impl Default for FilesystemState {
    fn default() -> Self {
        Self {
            filesystem: init_filesystem(),
        }
    }
}

thread_local! {
    static STATE: RefCell<FilesystemState> = RefCell::new(FilesystemState::default());
}
