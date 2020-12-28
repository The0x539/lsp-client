use serde::{ser::SerializeStruct, Serialize, Serializer};

use lsp_types::notification::Notification as NotificationTrait;
use lsp_types::request::Request as RequestTrait;

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

pub struct Response<R: RequestTrait> {
    pub id: u32,
    pub result: std::result::Result<R::Params, crate::error::ResponseError>,
}

impl<R: RequestTrait> Serialize for Response<R> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut s = serializer.serialize_struct("ResponseMessage", 3)?;
        s.serialize_field("jsonrpc", "2.0")?;
        s.serialize_field("id", &self.id)?;
        match &self.result {
            Ok(r) => s.serialize_field("result", r)?,
            Err(e) => s.serialize_field("error", e)?,
        }
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

mod private {
    use super::{Notification, Request, Response};
    use super::{NotificationTrait, RequestTrait};

    pub trait Sealed {}
    impl<R: RequestTrait> Sealed for Request<R> {}
    impl<R: RequestTrait> Sealed for Response<R> {}
    impl<N: NotificationTrait> Sealed for Notification<N> {}
    impl<T: Sealed> Sealed for &T {}
}
use private::Sealed;

pub trait Message: Serialize + Sealed {}
impl<R: RequestTrait> Message for Request<R> {}
impl<R: RequestTrait> Message for Response<R> {}
impl<N: NotificationTrait> Message for Notification<N> {}
impl<T: Message> Message for &T {}
