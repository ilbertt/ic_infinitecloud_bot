use std::{borrow::Cow, fmt, path::PathBuf};

use candid::{CandidType, Decode, Deserialize, Encode};
use ic_stable_structures::{storable::Bound, Storable};

use crate::{
    custom_print,
    utils::{
        filesystem::root_path,
        is_absolute,
        messages::{
            BACK_BUTTON_TEXT, CURRENT_DIR_BUTTON_TEXT, DELETE_DIR_BUTTON_TEXT, MKDIR_BUTTON_TEXT,
            PARENT_DIR_BUTTON_TEXT,
        },
    },
};

use super::FileSystemNode;

#[derive(Debug, CandidType, Deserialize, Clone, PartialEq, Eq)]
pub enum ChatSessionWaitReply {
    DirectoryName,
    FileName,
}

#[derive(Debug, CandidType, Deserialize, Clone, PartialEq, Eq)]
pub enum ChatSessionAction {
    MkDir(Option<ChatSessionWaitReply>),
    SaveFile(Option<FileSystemNode>, Option<ChatSessionWaitReply>),
    CurrentDir,
    ParentDir,
    DeleteDir,
    Explorer,
    RenameFile(Option<ChatSessionWaitReply>),
    MoveFile(Option<PathBuf>),
    DeleteFile,
    FileOrDir(PathBuf),
    Back,
}

impl ChatSessionAction {
    pub fn beautified(&self) -> String {
        match self {
            ChatSessionAction::MkDir(_) => MKDIR_BUTTON_TEXT.to_string(),
            ChatSessionAction::SaveFile(_, _) => "".to_string(),
            ChatSessionAction::CurrentDir => CURRENT_DIR_BUTTON_TEXT.to_string(),
            ChatSessionAction::ParentDir => PARENT_DIR_BUTTON_TEXT.to_string(),
            ChatSessionAction::DeleteDir => DELETE_DIR_BUTTON_TEXT.to_string(),
            ChatSessionAction::Explorer => "".to_string(),
            ChatSessionAction::RenameFile(_) => "".to_string(),
            ChatSessionAction::MoveFile(_) => "".to_string(),
            ChatSessionAction::DeleteFile => "".to_string(),
            ChatSessionAction::FileOrDir(path) => path.to_string_lossy().to_string(),
            ChatSessionAction::Back => BACK_BUTTON_TEXT.to_string(),
        }
    }
}

impl From<ChatSessionAction> for String {
    fn from(val: ChatSessionAction) -> Self {
        val.to_string()
    }
}

impl fmt::Display for ChatSessionAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ChatSessionAction::MkDir(_) => "mkdir-action".to_string(),
                ChatSessionAction::SaveFile(_, _) => "save-file-action".to_string(),
                ChatSessionAction::CurrentDir => ".".to_string(),
                ChatSessionAction::ParentDir => "..".to_string(),
                ChatSessionAction::DeleteDir => "delete-dir-action".to_string(),
                ChatSessionAction::Explorer => "explorer-action".to_string(),
                ChatSessionAction::RenameFile(_) => "rename-file-action".to_string(),
                ChatSessionAction::MoveFile(_) => "move-file-action".to_string(),
                ChatSessionAction::DeleteFile => "delete-file-action".to_string(),
                ChatSessionAction::FileOrDir(path) => path.to_string_lossy().to_string(),
                ChatSessionAction::Back => "back-action".to_string(),
            }
        )
    }
}

impl From<String> for ChatSessionAction {
    fn from(val: String) -> Self {
        match val.as_str() {
            "mkdir-action" => ChatSessionAction::MkDir(None),
            "save-file-action" => ChatSessionAction::SaveFile(None, None),
            "." => ChatSessionAction::CurrentDir,
            ".." => ChatSessionAction::ParentDir,
            "delete-dir-action" => ChatSessionAction::DeleteDir,
            "explorer-action" => ChatSessionAction::Explorer,
            "rename-file-action" => ChatSessionAction::RenameFile(None),
            "move-file-action" => ChatSessionAction::MoveFile(None),
            "delete-file-action" => ChatSessionAction::DeleteFile,
            "back-action" => ChatSessionAction::Back,
            _ => ChatSessionAction::FileOrDir(PathBuf::from(val)),
        }
    }
}

#[derive(Debug, CandidType, Deserialize, Clone, PartialEq, Eq)]
pub struct ChatSession {
    current_path: PathBuf,
    action: Option<ChatSessionAction>,
}

impl ChatSession {
    pub fn set_action(&mut self, action: ChatSessionAction) {
        self.action = Some(action)
    }

    pub fn action(&self) -> Option<ChatSessionAction> {
        self.action.clone()
    }

    pub fn clear_action(&mut self) {
        self.action = None
    }

    pub fn current_path(&self) -> &PathBuf {
        &self.current_path
    }

    pub fn current_path_string(&self) -> String {
        self.current_path.to_string_lossy().to_string()
    }

    pub fn set_current_path(&mut self, path: PathBuf) {
        if !is_absolute(&path) {
            panic!("Path is not absolute");
        }
        self.current_path = path
    }

    pub fn reset(&mut self) {
        self.set_current_path(root_path());
        self.action = None;
    }
}

impl Default for ChatSession {
    fn default() -> Self {
        Self {
            current_path: root_path(),
            action: None,
        }
    }
}

impl Storable for ChatSession {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Unbounded;
}

pub fn with_clear_action_on_error<F: FnOnce(&mut ChatSession) -> Result<R, String>, R>(
    chat_session: &mut ChatSession,
    f: F,
) -> Result<R, String> {
    let result = f(chat_session);
    if result.is_err() {
        custom_print!("An error occurred, clearing chat session action");
        chat_session.clear_action();
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[rstest]
    fn storable_impl() {
        let chat_session = ChatSession::default();

        let serialized_chat_session = chat_session.to_bytes();
        let deserialized_chat_session = ChatSession::from_bytes(serialized_chat_session);

        assert_eq!(deserialized_chat_session, chat_session);
    }

    #[rstest]
    fn set_current_path() {
        let mut chat_session = ChatSession::default();
        chat_session.set_current_path(PathBuf::from("/test"));
        assert_eq!(chat_session.current_path(), &PathBuf::from("/test"));
        chat_session.set_current_path(root_path());
        assert_eq!(chat_session.current_path(), &root_path());
    }

    #[rstest]
    #[should_panic(expected = "Path is not absolute")]
    fn set_current_path_relative() {
        let mut chat_session = ChatSession::default();
        chat_session.set_current_path(PathBuf::from("test"));
    }
}
