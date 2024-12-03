use crate::repositories::{ChatId, FileSystem, FilesystemRepository, FilesystemRepositoryImpl};

pub trait FilesystemService {
    fn get_or_create_filesystem(&self, chat_id: &ChatId) -> FileSystem;
    fn update_filesystem(&self, chat_id: &ChatId, filesystem: FileSystem);
}

pub struct FilesystemServiceImpl<T: FilesystemRepository> {
    filesystem_repository: T,
}

impl Default for FilesystemServiceImpl<FilesystemRepositoryImpl> {
    fn default() -> Self {
        Self::new(FilesystemRepositoryImpl::default())
    }
}

impl<T: FilesystemRepository> FilesystemService for FilesystemServiceImpl<T> {
    fn get_or_create_filesystem(&self, chat_id: &ChatId) -> FileSystem {
        match self
            .filesystem_repository
            .get_filesystem_by_chat_id(chat_id)
        {
            Some(filesystem) => filesystem,
            None => {
                let fs = FileSystem::default();
                self.filesystem_repository
                    .set_filesystem_by_chat_id(chat_id.clone(), fs.clone());
                fs
            }
        }
    }

    fn update_filesystem(&self, chat_id: &ChatId, filesystem: FileSystem) {
        self.filesystem_repository
            .set_filesystem_by_chat_id(chat_id.clone(), filesystem);
    }
}

impl<T: FilesystemRepository> FilesystemServiceImpl<T> {
    fn new(filesystem_repository: T) -> Self {
        Self {
            filesystem_repository,
        }
    }
}
