use serde::{ser::SerializeStruct, Deserialize, Serialize, Serializer};

use either::Either;
use lsp_types::notification::Notification as NotificationTrait;
use lsp_types::request::Request as RequestTrait;
use serde_json::Value;
use tokio::sync::oneshot;

pub struct Request<R: RequestTrait> {
    pub id: u32,
    pub params: R::Params,
}

impl<R: RequestTrait> Serialize for Request<R> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut s = serializer.serialize_struct("RequestMessage", 4)?;
        s.serialize_field("jsonrpc", "2.0")?;
        s.serialize_field("id", &self.id)?;
        s.serialize_field("method", R::METHOD)?;
        s.serialize_field("params", &self.params)?;
        s.end()
    }
}

pub struct Notification<N: NotificationTrait> {
    pub params: N::Params,
}

impl<N: NotificationTrait> Serialize for Notification<N> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut s = serializer.serialize_struct("NotificationMessage", 3)?;
        s.serialize_field("jsonrpc", "2.0")?;
        s.serialize_field("method", N::METHOD)?;
        s.serialize_field("params", &self.params)?;
        s.end()
    }
}

// TODO: seal this trait
pub trait OutboundMessage: Serialize {}
impl<R: RequestTrait> OutboundMessage for Request<R> {}
impl<N: NotificationTrait> OutboundMessage for Notification<N> {}
impl<T: OutboundMessage> OutboundMessage for &T {}

#[derive(Debug, Deserialize)]
pub struct Response<T> {
    pub id: u32,
    pub result: Option<T>,
    pub error: Option<crate::error::ResponseError>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IncomingNotification {
    pub method: String,
    pub params: Value,
}

pub type GenericResponse = Response<Value>;
pub type RpcSender = oneshot::Sender<GenericResponse>;
pub type Message = Either<GenericResponse, IncomingNotification>;
