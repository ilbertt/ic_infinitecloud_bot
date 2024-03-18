use std::{borrow::Cow, collections::BTreeMap, path::PathBuf};

use candid::{CandidType, Decode, Deserialize, Encode};
use frankenstein::ChatId as TgChatId;
use ic_stable_structures::{storable::Bound, Storable};

pub type MessageId = u64;

#[derive(Debug, CandidType, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChatId(pub u64);

impl From<i64> for ChatId {
    fn from(value: i64) -> Self {
        Self(value as u64)
    }
}

impl Into<TgChatId> for ChatId {
    fn into(self) -> TgChatId {
        TgChatId::Integer(self.0 as i64)
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
pub enum FileObject {
    File {
        message_id: MessageId,
        created_at: u64,
        size: u64,
    },
    Dir {
        created_at: u64,
    },
}

impl FileObject {
    fn new_file(message_id: MessageId, size: u64) -> Self {
        Self::File {
            message_id,
            created_at: 0,
            size,
        }
    }
    fn new_dir() -> Self {
        Self::Dir { created_at: 0 }
    }
}

#[derive(Debug, CandidType, Deserialize, Clone, PartialEq, Eq)]
pub struct Filesystem {
    pub objects: BTreeMap<PathBuf, FileObject>,
}

impl Default for Filesystem {
    fn default() -> Self {
        Self {
            objects: BTreeMap::from_iter(vec![
                (PathBuf::from("/"), FileObject::new_dir()),
                (PathBuf::from("/Documents"), FileObject::new_dir()),
                (PathBuf::from("/Images"), FileObject::new_dir()),
                (PathBuf::from("/Videos"), FileObject::new_dir()),
                (PathBuf::from("/Trash"), FileObject::new_dir()),
            ]),
        }
    }
}

impl Filesystem {
    pub fn ls(&self, path: &PathBuf) -> Vec<PathBuf> {
        self.objects
            .keys()
            .filter(|key| key.starts_with(path))
            .cloned()
            .collect()
    }

    pub fn mkdir(&mut self, path: &PathBuf) {
        self.objects.insert(path.to_owned(), FileObject::new_dir());
    }
}

impl Storable for Filesystem {
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
        let filesystem = Filesystem::default();

        let serialized_filesystem = filesystem.to_bytes();
        let deserialized_filesystem = Filesystem::from_bytes(serialized_filesystem);

        assert_eq!(deserialized_filesystem, filesystem);
    }
}
