use serde::Deserialize;

use serde_json::Value;
use tokio::sync::oneshot;

#[derive(Debug, Deserialize)]
pub struct Request<T> {
    pub id: u32,
    pub method: String,
    pub params: T,
}

#[derive(Debug, Deserialize)]
pub struct Response<T> {
    pub id: u32,
    pub result: Option<T>,
    pub error: Option<crate::error::ResponseError>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Notification {
    pub method: String,
    pub params: Value,
}

pub type GenericRequest = Request<Value>;
pub type GenericResponse = Response<Value>;
pub type RpcSender = oneshot::Sender<GenericResponse>;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Message {
    Request(GenericRequest),
    Response(GenericResponse),
    Notification(Notification),
}
