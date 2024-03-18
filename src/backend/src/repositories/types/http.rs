use candid::{define_function, CandidType, Deserialize};

#[derive(CandidType, Deserialize, Debug)]
pub struct HeaderField(pub String, pub String);

#[derive(CandidType, Deserialize, Debug)]
pub struct HttpRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<HeaderField>,
    pub body: Vec<u8>,
    pub certificate_version: Option<u16>,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct HttpUpdateRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<HeaderField>,
    pub body: Vec<u8>,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct HttpResponse {
    pub status_code: u16,
    pub headers: Vec<HeaderField>,
    pub body: Vec<u8>,
    pub upgrade: Option<bool>,
    pub streaming_strategy: Option<StreamingStrategy>,
}

pub type StreamingToken = String;

#[derive(CandidType, Deserialize, Debug)]
pub struct StreamingCallbackHttpResponse {
    pub body: Vec<u8>,
    pub token: Option<StreamingToken>,
}

define_function!(pub CallbackFunc : (StreamingToken) -> (StreamingCallbackHttpResponse) query);
#[derive(CandidType, Deserialize, Debug)]
pub enum StreamingStrategy {
    Callback {
        callback: CallbackFunc,
        token: StreamingToken,
    },
}
