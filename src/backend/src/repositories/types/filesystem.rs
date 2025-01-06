use std::{
    borrow::Cow,
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use candid::{CandidType, Decode, Deserialize, Encode};
use frankenstein::{InlineKeyboardButton, InlineKeyboardMarkup};
use ic_stable_structures::{storable::Bound, Storable};
use mime2ext::mime2ext;

use crate::utils::{
    filesystem::root_path,
    get_current_time, is_absolute,
    messages::{current_dir_inline_button, delete_dir_inline_button, parent_dir_inline_button},
    path_button, TG_FILE_MIME_TYPE_PREFIX,
};

pub type MessageId = i32;

#[derive(Debug, CandidType, Deserialize, Clone, PartialEq, Eq)]
pub enum FileSystemNode {
    File {
        message_id: MessageId,
        created_at: u64,
        size: u64,
        mime_type: Option<String>,
    },
    Directory {
        created_at: u64,
        nodes: FileSystemNodes,
    },
}

pub type FileSystemNodes = BTreeMap<PathBuf, FileSystemNode>;

impl FileSystemNode {
    pub fn new_file(message_id: MessageId, size: u64, mime_type: Option<String>) -> Self {
        Self::File {
            message_id,
            created_at: get_current_time(),
            size,
            mime_type,
        }
    }

    fn new_directory() -> Self {
        Self::Directory {
            created_at: get_current_time(),
            nodes: FileSystemNodes::new(),
        }
    }

    pub fn is_directory(&self) -> bool {
        matches!(self, Self::Directory { .. })
    }

    pub fn is_file(&self) -> bool {
        matches!(self, Self::File { .. })
    }

    #[cfg(test)]
    fn get_nodes(&self) -> &FileSystemNodes {
        if let Self::Directory { nodes, .. } = self {
            nodes
        } else {
            panic!("Not a directory");
        }
    }

    #[cfg(test)]
    fn get_nodes_mut(&mut self) -> &mut FileSystemNodes {
        if let Self::Directory { nodes, .. } = self {
            nodes
        } else {
            panic!("Not a directory");
        }
    }

    #[cfg(test)]
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

    pub fn file_message_id(&self) -> Option<MessageId> {
        if let Self::File { message_id, .. } = self {
            Some(*message_id)
        } else {
            None
        }
    }

    pub fn file_mime_type(&self) -> Option<String> {
        if let Self::File { mime_type, .. } = self {
            mime_type.clone()
        } else {
            None
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
    #[cfg(test)]
    fn new() -> Self {
        Self {
            root: FileSystemNode::new_directory(),
        }
    }

    pub fn get_node(&self, path: &Path) -> Result<&FileSystemNode, String> {
        if !is_absolute(path) {
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

    fn insert_node(&mut self, path: &Path, node: FileSystemNode) -> Result<(), String> {
        let parent = path.parent().ok_or("Invalid path")?;
        let mut current = &mut self.root;
        for component in parent.components().skip(1) {
            // Skip root path
            if let FileSystemNode::Directory { nodes, .. } = current {
                current = nodes
                    .entry(component.as_os_str().into())
                    .or_insert_with(FileSystemNode::new_directory);
            } else {
                return Err("Parent is not a directory".to_string());
            }
        }
        if let FileSystemNode::Directory { nodes, .. } = current {
            let new_node_key = path.file_name().ok_or("Invalid file name")?.into();
            nodes.insert(new_node_key, node);
            Ok(())
        } else {
            Err("Parent is not a directory".to_string())
        }
    }

    fn remove_node(&mut self, path: &Path) -> Result<FileSystemNode, String> {
        let parent = path.parent().ok_or("Invalid path")?;
        let mut current = &mut self.root;
        for component in parent.components().skip(1) {
            // Skip root path
            if let FileSystemNode::Directory { nodes, .. } = current {
                current = nodes
                    .entry(component.as_os_str().into())
                    .or_insert_with(FileSystemNode::new_directory);
            } else {
                return Err("Parent is not a directory".to_string());
            }
        }
        if let FileSystemNode::Directory { nodes, .. } = current {
            let node_key: PathBuf = path.file_name().ok_or("Invalid file name")?.into();
            nodes
                .remove(&node_key)
                .ok_or_else(|| "Node not found".to_string())
        } else {
            Err("Parent is not a directory".to_string())
        }
    }

    #[cfg(test)]
    fn ls(&self, path: &Path) -> Result<Vec<PathBuf>, String> {
        let node = self.get_node(path)?;
        if node.is_directory() {
            node.ls()
        } else {
            Err("Not a directory".to_string())
        }
    }

    pub fn mkdir(&mut self, path: &Path) -> Result<(), String> {
        self.insert_node(path, FileSystemNode::new_directory())
    }

    pub fn create_file_from_node(
        &mut self,
        path: &Path,
        file_node: FileSystemNode,
    ) -> Result<PathBuf, String> {
        let mut path = path.to_path_buf();

        if path.extension().is_none() {
            if let Some(mut ext) = file_node.file_mime_type() {
                if !ext.starts_with(TG_FILE_MIME_TYPE_PREFIX) {
                    ext = mime2ext(&ext).unwrap_or_default().to_string();
                }
                path = path.with_extension(ext);
            }
        }

        self.insert_node(&path, file_node)?;
        Ok(path)
    }

    #[cfg(test)]
    fn create_file(
        &mut self,
        path: &Path,
        message_id: MessageId,
        size: u64,
        mime_type: Option<String>,
    ) -> Result<PathBuf, String> {
        let file_node = FileSystemNode::new_file(message_id, size, mime_type);
        self.create_file_from_node(path, file_node)
    }

    pub fn mv(&mut self, from: &Path, to: &Path) -> Result<(), String> {
        let node = self.remove_node(from)?;
        self.insert_node(to, node)
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

pub struct KeyboardDirectoryBuilder<'a> {
    inline_keyboard: Vec<InlineKeyboardButton>,
    current_node: &'a FileSystemNode,
    current_path: &'a Path,
}

impl<'a> KeyboardDirectoryBuilder<'a> {
    pub fn new(filesystem: &'a FileSystem, current_path: &'a Path) -> Result<Self, String> {
        let current_node = filesystem.get_node(current_path)?;

        let mut inline_keyboard = if current_path != root_path() {
            vec![parent_dir_inline_button()]
        } else {
            vec![]
        };

        for path in current_node.ls_directories()? {
            inline_keyboard.push(path_button(&current_path.join(path), true));
        }

        Ok(Self {
            inline_keyboard,
            current_node,
            current_path,
        })
    }

    /// Prepends the delete dir button to the keyboard
    #[allow(dead_code)] // TODO: remove once used
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
            self.inline_keyboard
                .push(path_button(&self.current_path.join(path), false));
        }
        Ok(self)
    }

    pub fn build(&self) -> InlineKeyboardMarkup {
        InlineKeyboardMarkup {
            // to display one button per row, each button should have its own row
            inline_keyboard: self
                .inline_keyboard
                .iter()
                .map(|el| vec![el.clone()])
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

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
            .create_file(
                &PathBuf::from("/dir-a/file-a"),
                0,
                0,
                Some("text/plain".to_string()),
            )
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
            .create_file(
                &PathBuf::from("/dir-a/file-a"),
                0,
                0,
                Some("text/plain".to_string()),
            )
            .unwrap();
        filesystem
            .create_file(
                &PathBuf::from("/dir-b/file-b"),
                0,
                0,
                Some("image/png".to_string()),
            )
            .unwrap();
        filesystem
            .create_file(
                &PathBuf::from("/dir-b/dir-bb/file-bb"),
                0,
                0,
                Some("application/sql".to_string()),
            )
            .unwrap();
        filesystem
            .create_file(
                &PathBuf::from("/file-c"),
                0,
                0,
                Some("application/mp4".to_string()),
            )
            .unwrap();

        assert_eq!(
            filesystem.ls(&PathBuf::from("/")),
            Ok(vec![
                PathBuf::from("dir-a"),
                PathBuf::from("dir-b"),
                PathBuf::from("file-c.mp4")
            ])
        );

        assert_eq!(
            filesystem.ls(&PathBuf::from("/dir-a")),
            Ok(vec![PathBuf::from("file-a.txt")])
        );

        assert_eq!(
            filesystem.ls(&PathBuf::from("/dir-b")),
            Ok(vec![PathBuf::from("dir-bb"), PathBuf::from("file-b.png")])
        );

        assert_eq!(
            filesystem.ls(&PathBuf::from("/dir-b/dir-ba")),
            Err("Path not found".to_string())
        );

        assert_eq!(
            filesystem.ls(&PathBuf::from("/dir-b/dir-bb")),
            Ok(vec![PathBuf::from("file-bb.sql")])
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
            filesystem.ls(&PathBuf::from("/file-c.mp4")),
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
        let path = filesystem
            .create_file(
                &PathBuf::from("/dir-a/file-a"),
                0,
                0,
                Some("text/plain".to_string()),
            )
            .unwrap();
        let expected_path = PathBuf::from("/dir-a/file-a.txt");
        assert_eq!(path, expected_path);
        assert!(filesystem.get_node(&expected_path).unwrap().is_file());

        // preserve extension
        let path = filesystem
            .create_file(
                &PathBuf::from("/dir-a/file-b.mp3"),
                0,
                0,
                Some("text/plain".to_string()),
            )
            .unwrap();
        let expected_path = PathBuf::from("/dir-a/file-b.mp3");
        assert_eq!(path, expected_path);
        assert!(filesystem.get_node(&expected_path).unwrap().is_file());

        // do not parse tg+ mime types
        let path = filesystem
            .create_file(
                &PathBuf::from("/dir-a/file-c"),
                0,
                0,
                Some(format!("{TG_FILE_MIME_TYPE_PREFIX}video_note")),
            )
            .unwrap();
        let expected_path = PathBuf::from("/dir-a/file-c.tg+video_note");
        assert_eq!(path, expected_path);
        assert!(filesystem.get_node(&expected_path).unwrap().is_file());
    }

    #[rstest]
    fn filesystem_mv_directory() {
        let mut filesystem = FileSystem::new();
        filesystem.mkdir(&PathBuf::from("/dir-a/subdir-a")).unwrap();
        filesystem.mkdir(&PathBuf::from("/dir-b/subdir-b")).unwrap();
        filesystem
            .mv(
                &PathBuf::from("/dir-a/subdir-a"),
                &PathBuf::from("/dir-b/subdir-c"),
            )
            .unwrap();

        assert!(filesystem
            .get_node(&PathBuf::from("/dir-a/subdir-a"))
            .err()
            .unwrap()
            .contains("Path not found"));
        assert!(filesystem
            .get_node(&PathBuf::from("/dir-b/subdir-c"))
            .unwrap()
            .is_directory());
        // check that subdir-b is not moved
        assert!(filesystem
            .get_node(&PathBuf::from("/dir-b/subdir-b"))
            .unwrap()
            .is_directory());
    }

    #[rstest]
    fn filesystem_mv_file() {
        let mut filesystem = FileSystem::new();
        filesystem
            .create_file(
                &PathBuf::from("/dir-a/file-a"),
                0,
                0,
                Some("text/plain".to_string()),
            )
            .unwrap();
        filesystem
            .create_file(
                &PathBuf::from("/dir-b/file-b"),
                0,
                0,
                Some("image/png".to_string()),
            )
            .unwrap();
        filesystem
            .mv(
                &PathBuf::from("/dir-a/file-a.txt"),
                &PathBuf::from("/dir-b/file-a.txt"),
            )
            .unwrap();

        assert!(filesystem
            .get_node(&PathBuf::from("/dir-a/file-a.txt"))
            .err()
            .unwrap()
            .contains("Path not found"));
        assert!(filesystem
            .get_node(&PathBuf::from("/dir-b/file-a.txt"))
            .unwrap()
            .is_file());
        // check that file-b is not moved
        assert!(filesystem
            .get_node(&PathBuf::from("/dir-b/file-b.png"))
            .unwrap()
            .is_file());
    }

    #[rstest]
    fn filesystem_node_get_nodes() {
        let mut node = FileSystemNode::new_directory();
        node.get_nodes_mut().insert(
            PathBuf::from("file-a"),
            FileSystemNode::new_file(0, 0, None),
        );

        let nodes = node.get_nodes();

        assert_eq!(nodes.len(), 1);
    }

    #[rstest]
    #[should_panic(expected = "Not a directory")]
    fn filesystem_node_get_nodes_file_panic() {
        let node = FileSystemNode::new_file(0, 0, None);
        node.get_nodes();
    }

    #[rstest]
    fn filesystem_node_ls_directories() {
        let mut node = FileSystemNode::new_directory();
        node.get_nodes_mut()
            .insert(PathBuf::from("dir-a"), FileSystemNode::new_directory());
        node.get_nodes_mut().insert(
            PathBuf::from("file-a"),
            FileSystemNode::new_file(0, 0, None),
        );

        let directories = node.ls_directories().unwrap();

        assert_eq!(directories.len(), 1);
        assert_eq!(directories[0], PathBuf::from("dir-a"));
    }

    #[rstest]
    fn filesystem_node_ls_files() {
        let mut node = FileSystemNode::new_directory();
        node.get_nodes_mut().insert(
            PathBuf::from("file-a"),
            FileSystemNode::new_file(0, 0, Some("text/plain".to_string())),
        );

        let files = node.ls_files().unwrap();

        assert_eq!(files.len(), 1);
        assert_eq!(files[0], PathBuf::from("file-a"));
    }

    #[rstest]
    fn filesystem_node_is_directory() {
        let node = FileSystemNode::new_directory();
        assert!(node.is_directory());
        let node = FileSystemNode::new_file(0, 0, Some("text/plain".to_string()));
        assert!(!node.is_directory());
    }

    #[rstest]
    fn filesystem_node_is_file() {
        let node = FileSystemNode::new_file(0, 0, Some("text/plain".to_string()));
        assert!(node.is_file());
        let node = FileSystemNode::new_directory();
        assert!(!node.is_file());
    }

    #[rstest]
    fn keyboard_directory_builder_new() {
        let filesystem = FileSystem::default();
        let path = root_path();
        let builder = KeyboardDirectoryBuilder::new(&filesystem, &path).unwrap();

        let root_contents = filesystem.ls(&path).unwrap();
        assert_eq!(builder.inline_keyboard.len(), root_contents.len());
        for content_path in root_contents {
            assert!(builder
                .inline_keyboard
                .iter()
                .any(|button| { button == &path_button(&path.join(content_path.clone()), true) }));
        }
    }

    #[rstest]
    fn keyboard_directory_builder_new_non_root() {
        let mut filesystem = FileSystem::default();
        let path = PathBuf::from("/Documents");
        filesystem
            .create_file(&path.join("file-a"), 0, 0, Some("text/plain".to_string()))
            .unwrap();
        let builder = KeyboardDirectoryBuilder::new(&filesystem, &path).unwrap();

        let contents = filesystem
            .get_node(&path)
            .unwrap()
            .ls_directories()
            .unwrap();
        assert_eq!(builder.inline_keyboard.len(), contents.len() + 1);
        assert_eq!(builder.inline_keyboard[0], parent_dir_inline_button());
        for content_path in contents {
            assert!(builder
                .inline_keyboard
                .iter()
                .any(|button| { button == &path_button(&path.join(content_path.clone()), true) }));
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
            .create_file(
                &PathBuf::from("/test_file"),
                1,
                100,
                Some("text/plain".to_string()),
            )
            .unwrap();
        let path = PathBuf::from("/");
        let mut builder = KeyboardDirectoryBuilder::new(&filesystem, &path).unwrap();
        let keyboard = builder.with_files().unwrap().build();

        let file_paths = filesystem.ls(&path).unwrap();
        assert_eq!(keyboard.inline_keyboard.len(), file_paths.len());
    }
}
