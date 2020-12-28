use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize, Error)]
#[error("{self:?}")]
pub struct ResponseError {
    pub code: i32,
    pub message: String,
    pub data: Option<Value>,
}

#[derive(Debug, Copy, Clone, Error)]
pub enum ProtocolViolation {
    #[error("the response contained an error, but there was also a result member")]
    BothResultAndResponse,
    #[error("the response contained no error, but there was also no result member")]
    NeitherResultNorResponse,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("couldn't send message to client: {0}")]
    SendMsg(std::io::Error),
    #[error("couldn't receive message from client: {0}")]
    RecvMsg(std::io::Error),
    #[error("the response from the client contained an error: {0}")]
    Lsp(ResponseError),
    #[error("the client violated the LSP specification: {0}")]
    ProtocolViolation(#[from] ProtocolViolation),
}

pub type Result<T> = std::result::Result<T, Error>;
