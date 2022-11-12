use std::borrow::Cow;
use std::collections::HashMap;
use std::iter::Iterator;

use multimap::MultiMap;
use serde::de::value::Error as ValError;
use serde::de::{self, Deserialize, Error as DeError, IntoDeserializer};
use serde::forward_to_deserialize_any;
use serde_json::value::RawValue;

use crate::extract::metadata::{Source, SourceFormat, SourceFrom};
use crate::extract::Metadata;
use crate::http::form::FormData;
use crate::http::header::HeaderMap;
use crate::http::ParseError;
use crate::Request;

use super::{CowValue, VecValue};

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

    fn deserialize_value<T>(&mut self, seed: T) -> Result<T::Value, ValError>
    where
        T: de::DeserializeSeed<'de>,
    {
        let source = self
            .field_source
            .take()
            .expect("MapAccess::next_value called before next_key");

        if source.from == SourceFrom::Body && source.format == SourceFormat::Json {
            let value = self
                .field_str_value
                .expect("MapAccess::next_value called before next_key");
            let mut value = serde_json::Deserializer::new(serde_json::de::StrRead::new(value));
            seed.deserialize(&mut value)
                .map_err(|_| ValError::custom("pare value error"))
        } else if source.from == SourceFrom::Request {
            let field = self
                .metadata
                .fields
                .get(self.field_index as usize)
                .expect("Field must exist");
            let metadata = field.metadata.expect("Field's metadata must exist");
            seed.deserialize(RequestDeserializer {
                params: self.params,
                queries: self.queries,
                headers: self.headers,
                cookies: self.cookies,
                payload: self.payload.clone(),
                metadata,
                field_index: -1,
                field_source: None,
                field_str_value: None,
                field_vec_value: None,
            })
        } else if let Some(value) = self.field_str_value.take() {
            seed.deserialize(CowValue(value.into()))
        } else if let Some(value) = self.field_vec_value.take() {
            seed.deserialize(VecValue(value.into_iter()))
        } else {
            Err(ValError::custom("parse value error"))
        }
    }

    fn next(&mut self) -> Option<Cow<'_, str>> {
        if self.field_index < self.metadata.fields.len() as isize - 1 {
            self.field_index += 1;
            let field = &self.metadata.fields[self.field_index as usize];
            let sources = if !field.sources.is_empty() {
                &field.sources
            } else if !self.metadata.default_source.is_empty() {
                &self.metadata.default_source
            } else {
                tracing::error!("no sources for field {}", field.name);
                return None;
            };

            self.field_str_value = None;
            self.field_vec_value = None;
            let field_name: Cow<'_, str> = if let Some(rename_all) = self.metadata.rename_all {
                if let Some(rename) = field.rename {
                    Cow::from(rename)
                } else {
                    rename_all.rename(field.name).into()
                }
            } else {
                if let Some(rename) = field.rename {
                    rename
                } else {
                    field.name
                }
                .into()
            };

            for source in sources {
                match source.from {
                    SourceFrom::Request => {
                        self.field_source = Some(source);
                        return Some(Cow::from(field.name));
                    }
                    SourceFrom::Param => {
                        let mut value = self.params.get(&*field_name);
                        if value.is_none() {
                            for alias in &field.aliases {
                                value = self.params.get(*alias);
                                if value.is_some() {
                                    break;
                                }
                            }
                        }
                        if let Some(value) = value {
                            self.field_str_value = Some(value);
                            self.field_source = Some(source);
                            return Some(Cow::from(field.name));
                        }
                    }
                    SourceFrom::Query => {
                        let mut value = self.queries.get_vec(field_name.as_ref());
                        if value.is_none() {
                            for alias in &field.aliases {
                                value = self.queries.get_vec(*alias);
                                if value.is_some() {
                                    break;
                                }
                            }
                        }
                        if let Some(value) = value {
                            self.field_vec_value =
                                Some(value.iter().map(|v| CowValue(v.into())).collect());
                            self.field_source = Some(source);
                            return Some(Cow::from(field.name));
                        }
                    }
                    SourceFrom::Header => {
                        let mut value = None;
                        if self.headers.contains_key(field_name.as_ref()) {
                            value = Some(self.headers.get_all(field_name.as_ref()));
                        } else {
                            for alias in &field.aliases {
                                if self.headers.contains_key(*alias) {
                                    value = Some(self.headers.get_all(*alias));
                                    break;
                                }
                            }
                        };
                        if let Some(value) = value {
                            self.field_vec_value = Some(
                                value
                                    .iter()
                                    .map(|v| CowValue(Cow::from(v.to_str().unwrap_or_default())))
                                    .collect(),
                            );
                            self.field_source = Some(source);
                            return Some(Cow::from(field.name));
                        }
                    }
                    SourceFrom::Cookie => {
                        let mut value = None;
                        if let Some(cookie) = self.cookies.get(field_name.as_ref()) {
                            value = Some(cookie.value());
                        } else {
                            for alias in &field.aliases {
                                if let Some(cookie) = self.cookies.get(*alias) {
                                    value = Some(cookie.value());
                                    break;
                                }
                            }
                        };
                        if let Some(value) = value {
                            self.field_str_value = Some(value);
                            self.field_source = Some(source);
                            return Some(Cow::from(field.name));
                        }
                    }
                    SourceFrom::Body => match source.format {
                        SourceFormat::Json => {
                            if let Some(payload) = &self.payload {
                                match payload {
                                    Payload::FormData(form_data) => {
                                        let mut value = form_data.fields.get(field_name.as_ref());
                                        if value.is_none() {
                                            for alias in &field.aliases {
                                                value = form_data.fields.get(*alias);
                                                if value.is_some() {
                                                    break;
                                                }
                                            }
                                        }
                                        if let Some(value) = value {
                                            self.field_str_value = Some(value);
                                            self.field_source = Some(source);
                                            return Some(Cow::from(field.name));
                                        } else {
                                            return None;
                                        }
                                    }
                                    Payload::JsonMap(ref map) => {
                                        let mut value = map.get(field_name.as_ref());
                                        if value.is_none() {
                                            for alias in &field.aliases {
                                                value = map.get(alias);
                                                if value.is_some() {
                                                    break;
                                                }
                                            }
                                        }
                                        if let Some(value) = value {
                                            self.field_str_value = Some(value.get());
                                            self.field_source = Some(source);
                                            return Some(Cow::from(field.name));
                                        } else {
                                            return None;
                                        }
                                    }
                                    Payload::JsonStr(value) => {
                                        self.field_str_value = Some(*value);
                                        self.field_source = Some(source);
                                        return Some(Cow::from(field.name));
                                    }
                                }
                            } else {
                                return None;
                            }
                        }
                        SourceFormat::MultiMap => {
                            if let Some(Payload::FormData(form_data)) = self.payload {
                                let mut value = form_data.fields.get_vec(field.name);
                                if value.is_none() {
                                    for alias in &field.aliases {
                                        value = form_data.fields.get_vec(*alias);
                                        if value.is_some() {
                                            break;
                                        }
                                    }
                                }
                                if let Some(value) = value {
                                    self.field_vec_value = Some(
                                        value.iter().map(|v| CowValue(Cow::from(v))).collect(),
                                    );
                                    self.field_source = Some(source);
                                    return Some(Cow::from(field.name));
                                } else {
                                    return None;
                                }
                            } else {
                                return None;
                            }
                        }
                        _ => {
                            panic!("Unsupported source format: {:?}", source.format);
                        }
                    },
                }
            }
        }
        None
    }
}

impl<'de> de::Deserializer<'de> for RequestDeserializer<'de> {
    type Error = ValError;
    fn deserialize_any<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_map(&mut self)
    }
    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }
    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct tuple_struct map seq
        struct enum identifier ignored_any
    }
}

impl<'de> de::MapAccess<'de> for RequestDeserializer<'de> {
    type Error = ValError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        match self.next() {
            Some(key) => seed.deserialize(key.into_deserializer()).map(Some),
            None => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        self.deserialize_value(seed)
    }
}

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
#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    use crate::macros::Extractible;
    use crate::test::TestClient;

    #[tokio::test]
    async fn test_de_request_from_query() {
        #[derive(Deserialize, Extractible, Eq, PartialEq, Debug)]
        #[extract(internal, default_source(from = "query"))]
        struct RequestData {
            q1: String,
            q2: i64,
        }
        let mut req = TestClient::get("http://127.0.0.1:7878/test/1234/param2v")
            .query("q1", "q1v")
            .query("q2", "23")
            .build();
        let data: RequestData = req.extract().await.unwrap();
        assert_eq!(
            data,
            RequestData {
                q1: "q1v".to_string(),
                q2: 23
            }
        );
    }

    #[tokio::test]
    async fn test_de_request_with_lifetime() {
        #[derive(Deserialize, Extractible, Eq, PartialEq, Debug)]
        #[extract(internal, default_source(from = "query"))]
        struct RequestData<'a> {
            #[extract(source(from = "param"), source(from = "query"))]
            #[extract(source(from = "body"))]
            q1: &'a str,
            // #[extract(source(from = "query"))]
            // #[serde(alias = "param2", alias = "param3")]
            // q2: i64,
        }

        let mut req = TestClient::get("http://127.0.0.1:7878/test/1234/param2v")
            .query("q1", "q1v")
            .query("q2", "23")
            .build();
        let data: RequestData<'_> = req.extract().await.unwrap();
        assert_eq!(data, RequestData { q1: "q1v" });
    }

    #[tokio::test]
    async fn test_de_request_with_rename() {
        #[derive(Deserialize, Extractible, Eq, PartialEq, Debug)]
        #[extract(internal, default_source(from = "query"))]
        struct RequestData<'a> {
            #[extract(source(from = "param"), source(from = "query"), rename = "abc")]
            q1: &'a str,
        }

        let mut req = TestClient::get("http://127.0.0.1:7878/test/1234/param2v")
            .query("abc", "q1v")
            .build();
        let data: RequestData<'_> = req.extract().await.unwrap();
        assert_eq!(data, RequestData { q1: "q1v" });
    }

    #[tokio::test]
    async fn test_de_request_with_rename_all() {
        #[derive(Deserialize, Extractible, Eq, PartialEq, Debug)]
        #[extract(internal, default_source(from = "query"), rename_all = "PascalCase")]
        struct RequestData<'a> {
            first_name: &'a str,
            #[extract(rename = "lastName")]
            last_name: &'a str,
        }

        let mut req = TestClient::get("http://127.0.0.1:7878/test/1234/param2v")
            .query("FirstName", "chris")
            .query("lastName", "young")
            .build();
        let data: RequestData<'_> = req.extract().await.unwrap();
        assert_eq!(
            data,
            RequestData {
                first_name: "chris",
                last_name: "young"
            }
        );
    }

    #[tokio::test]
    async fn test_de_request_with_multi_sources() {
        #[derive(Deserialize, Extractible, Eq, PartialEq, Debug)]
        #[extract(internal, default_source(from = "query"))]
        struct RequestData<'a> {
            #[extract(source(from = "param"))]
            #[extract(alias = "param1")]
            p1: String,
            #[extract(source(from = "param"), alias = "param2")]
            p2: &'a str,
            #[extract(source(from = "param"), alias = "param3")]
            p3: usize,
            // #[extract(source(from = "query"))]
            q1: String,
            // #[extract(source(from = "query"))]
            q2: i64,
            // #[extract(source(from = "body", format = "json"))]
            // body: RequestBody<'a>,
        }

        let mut req = TestClient::get("http://127.0.0.1:7878/test/1234/param2v")
            .query("q1", "q1v")
            .query("q2", "23")
            .build();
        req.params.insert("param1".into(), "param1v".into());
        req.params.insert("p2".into(), "921".into());
        req.params.insert("p3".into(), "89785".into());
        let data: RequestData = req.extract().await.unwrap();
        assert_eq!(
            data,
            RequestData {
                p1: "param1v".into(),
                p2: "921",
                p3: 89785,
                q1: "q1v".into(),
                q2: 23
            }
        );
    }

    #[tokio::test]
    async fn test_de_request_with_json_vec() {
        #[derive(Deserialize, Extractible, Eq, PartialEq, Debug)]
        #[extract(internal, default_source(from = "body", format = "json"))]
        struct RequestData<'a> {
            #[extract(source(from = "param"))]
            p2: &'a str,
            users: Vec<User>,
        }
        #[derive(Deserialize, Serialize, Eq, PartialEq, Debug)]
        struct User {
            id: i64,
            name: String,
        }

        let mut req = TestClient::get("http://127.0.0.1:7878/test/1234/param2v")
            .json(&vec![
                User {
                    id: 1,
                    name: "chris".into(),
                },
                User {
                    id: 2,
                    name: "young".into(),
                },
            ])
            .build();
        req.params.insert("p2".into(), "921".into());
        let data: RequestData = req.extract().await.unwrap();
        assert_eq!(
            data,
            RequestData {
                p2: "921",
                users: vec![
                    User {
                        id: 1,
                        name: "chris".into()
                    },
                    User {
                        id: 2,
                        name: "young".into()
                    }
                ]
            }
        );
    }

    #[tokio::test]
    async fn test_de_request_with_json_bool() {
        #[derive(Deserialize, Extractible, Eq, PartialEq, Debug)]
        #[extract(internal, default_source(from = "body", format = "json"))]
        struct RequestData<'a> {
            #[extract(source(from = "param"))]
            p2: &'a str,
            b: bool,
        }

        let mut req = TestClient::get("http://127.0.0.1:7878/test/1234/param2v")
            .json(&true)
            .build();
        req.params.insert("p2".into(), "921".into());
        let data: RequestData = req.extract().await.unwrap();
        assert_eq!(data, RequestData { p2: "921", b: true });
    }
    #[tokio::test]
    async fn test_de_request_with_json_str() {
        #[derive(Deserialize, Extractible, Eq, PartialEq, Debug)]
        #[extract(internal, default_source(from = "body", format = "json"))]
        struct RequestData<'a> {
            #[extract(source(from = "param"))]
            p2: &'a str,
            s: &'a str,
        }

        let mut req = TestClient::get("http://127.0.0.1:7878/test/1234/param2v")
            .json(&"abcd-good")
            .build();
        req.params.insert("p2".into(), "921".into());
        let data: RequestData = req.extract().await.unwrap();
        assert_eq!(
            data,
            RequestData {
                p2: "921",
                s: "abcd-good"
            }
        );
    }
}
