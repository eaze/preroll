use std::fmt::{self, Display};
use std::str::FromStr;

use log::kv::{ToValue, Value};
use serde::de::{Error as DeError, Unexpected, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct CorrelationId {
    id: String,
}

impl CorrelationId {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let uuid = Uuid::new_v4();
        let buf = &mut [0; 36];
        let human_id = uuid.to_hyphenated().encode_lower(buf);
        Self {
            id: human_id.to_string(),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.id
    }
}

impl Display for CorrelationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

#[derive(Debug)]
pub enum Never {} // Similar to the ! / unstable Never type.

impl std::error::Error for Never {}

impl Display for Never {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        unreachable!()
    }
}

impl FromStr for CorrelationId {
    type Err = Never;

    fn from_str(string: &str) -> Result<Self, Never> {
        Ok(Self {
            id: string.to_string(),
        })
    }
}

struct CorrelationIdVisitor;

impl<'de> Visitor<'de> for CorrelationIdVisitor {
    type Value = CorrelationId;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "a UUID &str")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        match CorrelationId::from_str(v) {
            Ok(method) => Ok(method),
            Err(_) => Err(DeError::invalid_value(Unexpected::Str(v), &self)),
        }
    }
}

impl<'de> Deserialize<'de> for CorrelationId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(CorrelationIdVisitor)
    }
}

impl Serialize for CorrelationId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'v> ToValue for CorrelationId {
    fn to_value(&self) -> Value<'_> {
        Value::from(self.as_str())
    }
}
