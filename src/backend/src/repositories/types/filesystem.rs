use std::{
    borrow::Cow,
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use candid::{CandidType, Decode, Deserialize, Encode};
use frankenstein::{ChatId as TgChatId, InlineKeyboardButton, InlineKeyboardMarkup};
use ic_stable_structures::{storable::Bound, Storable};

use crate::utils::{
    filesystem::root_path,
    messages::{current_dir_inline_button, delete_dir_inline_button, parent_dir_inline_button},
};

use super::ChatSessionAction;

pub type MessageId = u64;

#[derive(Debug, CandidType, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChatId(pub u64);

impl From<i64> for ChatId {
    fn from(value: i64) -> Self {
        Self(value as u64)
    }
}

impl From<ChatId> for TgChatId {
    fn from(val: ChatId) -> Self {
        TgChatId::Integer(val.0 as i64)
    }
}

impl ChatId {
    pub fn into_tg_chat_id(self) -> TgChatId {
        self.into()
    }
}

impl Storable for ChatId {
    fn to_bytes(&self) -> Cow<[u8]> {
        self.0.to_bytes()
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Self(u64::from_bytes(bytes))
    }

    const BOUND: Bound = u64::BOUND;
}

#[derive(Debug, CandidType, Deserialize, Clone, PartialEq, Eq)]
pub enum FileSystemNode {
    File {
        message_id: MessageId,
        created_at: u64,
        size: u64,
    },
    Directory {
        created_at: u64,
        nodes: FileSystemNodes,
    },
}

pub type FileSystemNodes = BTreeMap<PathBuf, FileSystemNode>;

impl FileSystemNode {
    fn new_file(message_id: MessageId, size: u64) -> Self {
        Self::File {
            message_id,
            created_at: 0, // TODO: add current timestamp
            size,
        }
    }

    fn new_directory() -> Self {
        Self::Directory {
            created_at: 0, // TODO: add current timestamp
            nodes: FileSystemNodes::new(),
        }
    }

    pub fn is_directory(&self) -> bool {
        matches!(self, Self::Directory { .. })
    }

    pub fn is_file(&self) -> bool {
        matches!(self, Self::File { .. })
    }

    fn get_nodes(&self) -> &FileSystemNodes {
        if let Self::Directory { nodes, .. } = self {
            nodes
        } else {
            panic!("Not a directory");
        }
    }

    fn get_nodes_mut(&mut self) -> &mut FileSystemNodes {
        if let Self::Directory { nodes, .. } = self {
            nodes
        } else {
            panic!("Not a directory");
        }
    }

    fn ls(&self) -> Result<Vec<PathBuf>, String> {
        match self {
            Self::Directory { nodes, .. } => Ok(nodes.keys().cloned().collect()),
            Self::File { .. } => Err("Not a directory".to_string()),
        }
    }

    fn ls_directories(&self) -> Result<Vec<PathBuf>, String> {
        match self {
            Self::Directory { nodes, .. } => {
                let mut result = Vec::new();
                for (path, node) in nodes {
                    if node.is_directory() {
                        result.push(path.clone());
                    }
                }
                Ok(result)
            }
            Self::File { .. } => Err("Not a directory".to_string()),
        }
    }

    fn ls_files(&self) -> Result<Vec<PathBuf>, String> {
        match self {
            Self::Directory { nodes, .. } => {
                let mut result = Vec::new();
                for (path, node) in nodes {
                    if node.is_file() {
                        result.push(path.clone());
                    }
                }
                Ok(result)
            }
            Self::File { .. } => Err("Not a directory".to_string()),
        }
    }
}

#[derive(Debug, CandidType, Deserialize, Clone, PartialEq, Eq)]
pub struct FileSystem {
    root: FileSystemNode,
}

impl Default for FileSystem {
    fn default() -> Self {
        let mut root = FileSystemNode::new_directory();
        if let FileSystemNode::Directory { ref mut nodes, .. } = root {
            nodes.insert(PathBuf::from("Documents"), FileSystemNode::new_directory());
            nodes.insert(PathBuf::from("Images"), FileSystemNode::new_directory());
            nodes.insert(PathBuf::from("Videos"), FileSystemNode::new_directory());
            nodes.insert(PathBuf::from("Trash"), FileSystemNode::new_directory());
        }
        Self { root }
    }
}

impl FileSystem {
    pub fn new() -> Self {
        Self {
            root: FileSystemNode::new_directory(),
        }
    }

    pub fn get_node(&self, path: &Path) -> Result<&FileSystemNode, String> {
        if !path.is_absolute() {
            return Err("Path must be absolute".to_string());
        }

        let mut current = &self.root;
        for component in path.components().skip(1) {
            // Skip root
            if let FileSystemNode::Directory { nodes, .. } = current {
                current = nodes
                    .get::<Path>(component.as_ref())
                    .ok_or("Path not found")?;
            } else {
                return Ok(current);
            }
        }
        Ok(current)
    }

    pub fn ls(&self, path: &Path) -> Result<Vec<PathBuf>, String> {
        let node = self.get_node(path)?;
        if node.is_directory() {
            node.ls()
        } else {
            Err("Not a directory".to_string())
        }
    }

    pub fn mkdir(&mut self, path: &Path) -> Result<(), String> {
        let parent = path.parent().ok_or("Invalid path")?;
        let new_dir_name = path.file_name().ok_or("Invalid directory name")?;

        let mut current = &mut self.root;
        for component in parent.components().skip(1) {
            // Skip root
            if let FileSystemNode::Directory { nodes, .. } = current {
                current = nodes
                    .entry(component.as_os_str().into())
                    .or_insert_with(FileSystemNode::new_directory);
            } else {
                return Err("Parent is not a directory".to_string());
            }
        }

        if let FileSystemNode::Directory { nodes, .. } = current {
            nodes.insert(new_dir_name.into(), FileSystemNode::new_directory());
            Ok(())
        } else {
            Err("Parent is not a directory".to_string())
        }
    }

    pub fn create_file(
        &mut self,
        path: &Path,
        message_id: MessageId,
        size: u64,
    ) -> Result<(), String> {
        let parent = path.parent().ok_or("Invalid path")?;
        let file_name = path.file_name().ok_or("Invalid file name")?;

        let mut current = &mut self.root;
        for component in parent.components().skip(1) {
            // Skip root
            if let FileSystemNode::Directory { nodes, .. } = current {
                current = nodes
                    .entry(component.as_os_str().into())
                    .or_insert_with(FileSystemNode::new_directory);
            } else {
                return Err("Parent is not a directory".to_string());
            }
        }

        if let FileSystemNode::Directory { nodes, .. } = current {
            nodes.insert(file_name.into(), FileSystemNode::new_file(message_id, size));
            Ok(())
        } else {
            Err("Parent is not a directory".to_string())
        }
    }
}

impl Storable for FileSystem {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Unbounded;
}

fn path_button(path: &Path) -> InlineKeyboardButton {
    InlineKeyboardButton::builder()
        .text(path.display().to_string())
        .callback_data(ChatSessionAction::FileOrDir(path.to_path_buf()))
        .build()
}

pub struct KeyboardDirectoryBuilder<'a> {
    inline_keyboard: Vec<InlineKeyboardButton>,
    current_node: &'a FileSystemNode,
}

impl<'a> KeyboardDirectoryBuilder<'a> {
    pub fn new(filesystem: &'a FileSystem, current_path: &Path) -> Result<Self, String> {
        let current_node = filesystem.get_node(current_path)?;

        let mut inline_keyboard = if current_path != root_path() {
            vec![parent_dir_inline_button()]
        } else {
            vec![]
        };

        for path in current_node.ls_directories()? {
            inline_keyboard.push(path_button(&path));
        }

        Ok(Self {
            inline_keyboard,
            current_node,
        })
    }

    /// Prepends the delete dir button to the keyboard
    pub fn with_delete_dir_button(&mut self) -> &mut Self {
        self.inline_keyboard.insert(0, delete_dir_inline_button());
        self
    }

    /// Prepends the current dir button to the keyboard
    pub fn with_current_dir_button(&mut self) -> &mut Self {
        self.inline_keyboard.insert(0, current_dir_inline_button());
        self
    }

    /// Appends the files of the current directory to the keyboard
    pub fn with_files(&mut self) -> Result<&mut Self, String> {
        let paths = self.current_node.ls_files()?;
        for path in paths {
            self.inline_keyboard.push(path_button(&path));
        }
        Ok(self)
    }

    pub fn build(&self) -> InlineKeyboardMarkup {
        InlineKeyboardMarkup {
            inline_keyboard: vec![self.inline_keyboard.clone()],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[rstest]
    fn chat_id_storable_impl() {
        let chat_id = ChatId(123);

        let serialized_chat_id = chat_id.to_bytes();
        let deserialized_chat_id = ChatId::from_bytes(serialized_chat_id);

        assert_eq!(deserialized_chat_id, chat_id);
    }

    #[rstest]
    fn chat_id_from() {
        let from: ChatId = 123i64.into();

        assert_eq!(from, ChatId(123));
    }

    #[rstest]
    fn into_tg_chat_id() {
        let chat_id = ChatId(123);
        let tg_chat_id: TgChatId = chat_id.into_tg_chat_id();

        assert_eq!(tg_chat_id, TgChatId::Integer(123));
    }

    #[rstest]
    fn filesystem_storable_impl() {
        let filesystem = FileSystem::default();

        let serialized_filesystem = filesystem.to_bytes();
        let deserialized_filesystem = FileSystem::from_bytes(serialized_filesystem);

        assert_eq!(deserialized_filesystem, filesystem);
    }

    #[rstest]
    fn filesystem_get_node_directory() {
        let filesystem = FileSystem::default();

        let node = filesystem.get_node(&PathBuf::from("/Documents")).unwrap();
        assert!(node.is_directory());
        let node = filesystem.get_node(&root_path()).unwrap();
        assert!(node.is_directory());
    }

    #[rstest]
    fn filesystem_get_node_file() {
        let mut filesystem = FileSystem::default();
        filesystem
            .create_file(&PathBuf::from("/dir-a/file-a.txt"), 0, 0)
            .unwrap();

        let node = filesystem
            .get_node(&PathBuf::from("/dir-a/file-a.txt"))
            .unwrap();
        assert!(node.is_file());
    }

    #[rstest]
    fn filesystem_get_node_not_found() {
        let filesystem = FileSystem::default();

        let node = filesystem.get_node(&PathBuf::from("/non-existent"));

        assert_eq!(node, Err("Path not found".to_string()));
    }

    #[rstest]
    fn filesystem_get_node_not_absolute() {
        let filesystem = FileSystem::default();

        let node = filesystem.get_node(&PathBuf::from("Documents"));

        assert_eq!(node, Err("Path must be absolute".to_string()));
    }

    #[rstest]
    fn filesystem_ls() {
        let mut filesystem = FileSystem::new();
        filesystem
            .create_file(&PathBuf::from("/dir-a/file-a.txt"), 0, 0)
            .unwrap();
        filesystem
            .create_file(&PathBuf::from("/dir-b/file-b.txt"), 0, 0)
            .unwrap();
        filesystem
            .create_file(&PathBuf::from("/dir-b/dir-bb/file-bb.txt"), 0, 0)
            .unwrap();
        filesystem
            .create_file(&PathBuf::from("/file-c.txt"), 0, 0)
            .unwrap();

        assert_eq!(
            filesystem.ls(&PathBuf::from("/")),
            Ok(vec![
                PathBuf::from("dir-a"),
                PathBuf::from("dir-b"),
                PathBuf::from("file-c.txt")
            ])
        );

        assert_eq!(
            filesystem.ls(&PathBuf::from("/dir-a")),
            Ok(vec![PathBuf::from("file-a.txt")])
        );

        assert_eq!(
            filesystem.ls(&PathBuf::from("/dir-b")),
            Ok(vec![PathBuf::from("dir-bb"), PathBuf::from("file-b.txt")])
        );

        assert_eq!(
            filesystem.ls(&PathBuf::from("/dir-b/dir-ba")),
            Err("Path not found".to_string())
        );

        assert_eq!(
            filesystem.ls(&PathBuf::from("/dir-b/dir-bb")),
            Ok(vec![PathBuf::from("file-bb.txt")])
        );

        assert_eq!(
            filesystem.ls(&PathBuf::from("dir-a")),
            Err("Path must be absolute".to_string())
        );

        assert_eq!(
            filesystem.ls(&PathBuf::from("/non-existent")),
            Err("Path not found".to_string())
        );
        assert_eq!(
            filesystem.ls(&PathBuf::from("/non-existent/non-existent")),
            Err("Path not found".to_string())
        );

        assert_eq!(
            filesystem.ls(&PathBuf::from("/dir-a/file-a.txt")),
            Err("Not a directory".to_string())
        );
        assert_eq!(
            filesystem.ls(&PathBuf::from("/file-c.txt")),
            Err("Not a directory".to_string())
        );
        assert_eq!(
            filesystem.ls(&PathBuf::from("/file-d.txt")),
            Err("Path not found".to_string())
        );
    }

    #[rstest]
    fn filesystem_mkdir() {
        let mut filesystem = FileSystem::new();
        filesystem.mkdir(&PathBuf::from("/dir-a")).unwrap();

        assert!(filesystem
            .get_node(&PathBuf::from("/dir-a"))
            .unwrap()
            .is_directory());
    }

    #[rstest]
    fn filesystem_create_file() {
        let mut filesystem = FileSystem::new();
        filesystem
            .create_file(&PathBuf::from("/dir-a/file-a.txt"), 0, 0)
            .unwrap();

        assert!(filesystem
            .get_node(&PathBuf::from("/dir-a/file-a.txt"))
            .unwrap()
            .is_file());
    }

    #[rstest]
    fn filesystem_node_get_nodes() {
        let mut node = FileSystemNode::new_directory();
        node.get_nodes_mut()
            .insert(PathBuf::from("file-a.txt"), FileSystemNode::new_file(0, 0));

        let nodes = node.get_nodes();

        assert_eq!(nodes.len(), 1);
    }

    #[rstest]
    #[should_panic(expected = "Not a directory")]
    fn filesystem_node_get_nodes_file_panic() {
        let node = FileSystemNode::new_file(0, 0);
        node.get_nodes();
    }

    #[rstest]
    fn filesystem_node_ls_directories() {
        let mut node = FileSystemNode::new_directory();
        node.get_nodes_mut()
            .insert(PathBuf::from("dir-a"), FileSystemNode::new_directory());
        node.get_nodes_mut()
            .insert(PathBuf::from("file-a.txt"), FileSystemNode::new_file(0, 0));

        let directories = node.ls_directories().unwrap();

        assert_eq!(directories.len(), 1);
        assert_eq!(directories[0], PathBuf::from("dir-a"));
    }

    #[rstest]
    fn filesystem_node_ls_files() {
        let mut node = FileSystemNode::new_directory();
        node.get_nodes_mut()
            .insert(PathBuf::from("file-a.txt"), FileSystemNode::new_file(0, 0));

        let files = node.ls_files().unwrap();

        assert_eq!(files.len(), 1);
        assert_eq!(files[0], PathBuf::from("file-a.txt"));
    }

    #[rstest]
    fn filesystem_node_is_directory() {
        let node = FileSystemNode::new_directory();
        assert!(node.is_directory());
        let node = FileSystemNode::new_file(0, 0);
        assert!(!node.is_directory());
    }

    #[rstest]
    fn filesystem_node_is_file() {
        let node = FileSystemNode::new_file(0, 0);
        assert!(node.is_file());
        let node = FileSystemNode::new_directory();
        assert!(!node.is_file());
    }

    #[rstest]
    fn keyboard_directory_builder_new() {
        let filesystem = FileSystem::default();
        let builder = KeyboardDirectoryBuilder::new(&filesystem, &PathBuf::from("/")).unwrap();

        let root_contents = filesystem.ls(&PathBuf::from("/")).unwrap();
        assert_eq!(builder.inline_keyboard.len(), root_contents.len());
        for path in root_contents {
            assert!(builder
                .inline_keyboard
                .iter()
                .any(|button| { button == &path_button(&path) }));
        }
    }

    #[rstest]
    fn keyboard_directory_builder_new_non_root() {
        let mut filesystem = FileSystem::default();
        let path = PathBuf::from("/Documents");
        filesystem
            .create_file(&path.join("file-a.txt"), 0, 0)
            .unwrap();
        let builder = KeyboardDirectoryBuilder::new(&filesystem, &path).unwrap();

        let contents = filesystem
            .get_node(&path)
            .unwrap()
            .ls_directories()
            .unwrap();
        assert_eq!(builder.inline_keyboard.len(), contents.len() + 1);
        assert_eq!(builder.inline_keyboard[0], parent_dir_inline_button());
        for path in contents {
            assert!(builder
                .inline_keyboard
                .iter()
                .any(|button| { button == &path_button(&path) }));
        }
    }

    #[rstest]
    fn test_keyboard_directory_builder_with_current_dir_button() {
        let filesystem = FileSystem::default();
        let path = PathBuf::from("/");
        let mut builder = KeyboardDirectoryBuilder::new(&filesystem, &path).unwrap();
        let keyboard = builder.with_current_dir_button().build();

        assert_eq!(keyboard.inline_keyboard[0][0], current_dir_inline_button());
    }

    #[rstest]
    fn test_keyboard_directory_builder_with_delete_dir_button() {
        let filesystem = FileSystem::default();
        let path = PathBuf::from("/");
        let mut builder = KeyboardDirectoryBuilder::new(&filesystem, &path).unwrap();
        let keyboard = builder.with_delete_dir_button().build();

        assert_eq!(keyboard.inline_keyboard[0][0], delete_dir_inline_button());
    }

    #[rstest]
    fn test_keyboard_directory_builder_with_files() {
        let mut filesystem = FileSystem::default();
        filesystem
            .create_file(&PathBuf::from("/test_file.txt"), 1, 100)
            .unwrap();
        let path = PathBuf::from("/");
        let mut builder = KeyboardDirectoryBuilder::new(&filesystem, &path).unwrap();
        let keyboard = builder.with_files().unwrap().build();

        let file_paths = filesystem.ls(&path).unwrap();
        assert_eq!(keyboard.inline_keyboard[0].len(), file_paths.len());
        for file_path in file_paths {
            assert!(keyboard.inline_keyboard[0]
                .iter()
                .any(|button| button == &path_button(&file_path)));
        }
    }
}
