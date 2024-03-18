use std::borrow::Cow;

use candid::{CandidType, Decode, Deserialize, Encode};
use ic_stable_structures::{storable::Bound, Storable};

#[derive(Debug, CandidType, Deserialize, Clone, PartialEq, Eq)]
pub struct ChatSession {}

impl Default for ChatSession {
    fn default() -> Self {
        Self {}
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
