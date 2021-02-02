use std::fmt::{self, Display};
use std::str::FromStr;

use log::kv::{ToValue, Value};
use serde::de::{Error as DeError, Unexpected, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct RequestId {
    id: Uuid,
    string_id: String,
}

impl RequestId {
    #[allow(clippy::new_without_default)]
    #[cfg(not(feature = "test"))]
    pub fn new() -> Self {
        Uuid::new_v4().into()
    }

    pub fn as_str(&self) -> &str {
        &self.string_id
    }

    #[cfg(feature = "honeycomb")]
    #[cfg_attr(feature = "docs", doc(cfg(feature = "honeycomb")))]
    pub fn as_u128(&self) -> u128 {
        self.id.as_u128()
    }
}

impl Display for RequestId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl From<Uuid> for RequestId {
    fn from(uuid: Uuid) -> Self {
        let buf = &mut [0; 36];
        let human_id = uuid.to_hyphenated().encode_lower(buf);
        Self {
            id: uuid,
            string_id: human_id.to_string(),
        }
    }
}

impl FromStr for RequestId {
    type Err = uuid::Error;

    fn from_str(string: &str) -> Result<Self, uuid::Error> {
        Ok(Self {
            id: Uuid::parse_str(string)?,
            string_id: string.to_string(),
        })
    }
}

struct RequestIdVisitor;

impl<'de> Visitor<'de> for RequestIdVisitor {
    type Value = RequestId;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "a UUID &str")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        match RequestId::from_str(v) {
            Ok(method) => Ok(method),
            Err(_) => Err(DeError::invalid_value(Unexpected::Str(v), &self)),
        }
    }
}

impl<'de> Deserialize<'de> for RequestId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(RequestIdVisitor)
    }
}

impl Serialize for RequestId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'v> ToValue for RequestId {
    fn to_value(&self) -> Value<'_> {
        Value::from(self.as_str())
    }
}
