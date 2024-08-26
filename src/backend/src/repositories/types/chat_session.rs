use std::{borrow::Cow, fmt, path::PathBuf};

use candid::{CandidType, Decode, Deserialize, Encode};
use ic_stable_structures::{storable::Bound, Storable};

use crate::utils::{
    filesystem::root_path,
    messages::{
        CURRENT_DIR_BUTTON_TEXT, DELETE_DIR_BUTTON_TEXT, MKDIR_BUTTON_TEXT, PARENT_DIR_BUTTON_TEXT,
    },
};

#[derive(Debug, CandidType, Deserialize, Clone, PartialEq, Eq)]
pub enum ChatSessionAction {
    MkDir,
    CurrentDir,
    ParentDir,
    DeleteDir,
    Explorer,
    RenameFile,
    PrepareMoveFile,
    DeleteFile,
}

impl ChatSessionAction {
    pub fn beautified(&self) -> String {
        match self {
            ChatSessionAction::MkDir => MKDIR_BUTTON_TEXT.to_string(),
            ChatSessionAction::CurrentDir => CURRENT_DIR_BUTTON_TEXT.to_string(),
            ChatSessionAction::ParentDir => PARENT_DIR_BUTTON_TEXT.to_string(),
            ChatSessionAction::DeleteDir => DELETE_DIR_BUTTON_TEXT.to_string(),
            ChatSessionAction::Explorer => "".to_string(),
            ChatSessionAction::RenameFile => "".to_string(),
            ChatSessionAction::PrepareMoveFile => "".to_string(),
            ChatSessionAction::DeleteFile => "".to_string(),
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
                ChatSessionAction::MkDir => "mkdir-action".to_string(),
                ChatSessionAction::CurrentDir => ".".to_string(),
                ChatSessionAction::ParentDir => "..".to_string(),
                ChatSessionAction::DeleteDir => "delete-dir-action".to_string(),
                ChatSessionAction::Explorer => "explorer-action".to_string(),
                ChatSessionAction::RenameFile => "rename-file-action".to_string(),
                ChatSessionAction::PrepareMoveFile => "prepare-move-file-action".to_string(),
                ChatSessionAction::DeleteFile => "delete-file-action".to_string(),
            }
        )
    }
}

impl From<String> for ChatSessionAction {
    fn from(val: String) -> Self {
        match val.as_str() {
            "mkdir-action" => ChatSessionAction::MkDir,
            "." => ChatSessionAction::CurrentDir,
            ".." => ChatSessionAction::ParentDir,
            "delete-dir-action" => ChatSessionAction::DeleteDir,
            "explorer-action" => ChatSessionAction::Explorer,
            "rename-file-action" => ChatSessionAction::RenameFile,
            "prepare-move-file-action" => ChatSessionAction::PrepareMoveFile,
            "delete-file-action" => ChatSessionAction::DeleteFile,
            _ => todo!(),
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
        self.current_path = path
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
}
