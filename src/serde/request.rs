use std::collections::HashMap;

use hyper::HeaderMap;
use multimap::MultiMap;
use serde_json::value::RawValue;

use serde::de::value::Error as ValError;
use serde::de::{self, Deserialize, Error as DeError, IntoDeserializer};
use serde::forward_to_deserialize_any;

use crate::{
    extract::{Metadata, Source},
    http::{errors::ParseError, form::FormData, request::Request},
};

use super::CowValue;

#[derive(Debug, Clone)]
pub(crate) enum Payload<'a> {
    FormData(&'a FormData),
    JsonStr(&'a str),
    JsonMap(HashMap<&'a str, &'a RawValue>),
}

#[derive(Debug)]
pub(crate) struct RequestDeserializer<'de> {
    params: &'de HashMap<String, String>,
    queries: &'de MultiMap<String, String>,
    cookies: &'de cookie::CookieJar,
    headers: &'de HeaderMap,
    payload: Option<Payload<'de>>,
    metadata: &'de Metadata,
    field_index: isize,
    field_source: Option<&'de Source>,
    field_str_value: Option<&'de str>,
    field_vec_value: Option<Vec<CowValue<'de>>>,
}

impl<'de> RequestDeserializer<'de> {
    pub(crate) fn new(
        request: &'de mut Request,
        metadata: &'de Metadata,
    ) -> Result<RequestDeserializer<'de>, ParseError> {
        let mut payload = None;
        if let Some(ctype) = request.content_type() {
            match ctype.subtype() {
                mime::WWW_FORM_URLENCODED | mime::FORM_DATA => {
                    payload = request.form_data.get().map(Payload::FormData);
                }
                mime::JSON => {
                    if let Some(data) = request.payload.get() {
                        payload = match serde_json::from_slice::<HashMap<&str, &RawValue>>(data) {
                            Ok(map) => Some(Payload::JsonMap(map)),
                            Err(_) => Some(Payload::JsonStr(std::str::from_utf8(data)?)),
                        };
                    }
                }
                _ => {}
            }
        }
        Ok(RequestDeserializer {
            params: request.params(),
            queries: request.queries(),
            cookies: request.cookies(),
            headers: request.headers(),
            payload,
            metadata,
            field_index: -1,
            field_source: None,
            field_str_value: None,
            field_vec_value: None,
        })
    }
}

impl<'de> de::Deserializer<'de> for RequestDeserializer<'de> {
    type Error = ValError;
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: de::Visitor<'de> {
        self.deserialize_any(visitor)
    }
    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: de::Visitor<'de> {
        self.deserialize_seq(visitor)
    }
    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct tuple_struct map seq
        struct enum identifier ignored_any
    }
}

impl<>

pub(crate) async fn from_request<'de, T>(
    req: &'de mut Request,
    metadata: &'de Metadata,
) -> Result<T, ParseError>
where
    T: Deserialize<'de>,
{
    req.form_data().await.ok();
    req.payload().await.ok();
    Ok(T::deserialize(RequestDeserializer::new(req, metadata)?)?)
}
