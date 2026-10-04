//! Serde helpers for PHP-isms in Nextcloud requests and responses.

use std::collections::HashMap;

use serde::{Deserialize, Deserializer, Serializer, de::DeserializeOwned};

/// Many query parameters are `0|1` instead of booleans.
pub fn bool_as_int<S: Serializer>(value: &bool, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_u8(u8::from(*value))
}

/// PHP serializes an empty associative array as `[]`, and some endpoints answer
/// `{}` for "nothing". Treat `[]`, `{}` and `null` as `T::default()` and
/// deserialize anything else as `T`.
pub fn empty_as_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: DeserializeOwned + Default,
{
    Ok(empty_as_none(deserializer)?.unwrap_or_default())
}

/// Like [`empty_as_default`], but yields `None` for `[]`, `{}` and `null`.
pub fn empty_as_none<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: DeserializeOwned,
{
    match serde_json::Value::deserialize(deserializer)? {
        serde_json::Value::Null => Ok(None),
        serde_json::Value::Array(items) if items.is_empty() => Ok(None),
        serde_json::Value::Object(fields) if fields.is_empty() => Ok(None),
        value => serde_json::from_value(value)
            .map(Some)
            .map_err(serde::de::Error::custom),
    }
}

/// A map in a position where the field helpers can't be used (e.g. OCS `data`),
/// tolerating `[]` for an empty map.
#[derive(Debug, Clone, Default)]
pub struct PhpMap<V>(pub HashMap<String, V>);

impl<'de, V: DeserializeOwned> Deserialize<'de> for PhpMap<V> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        empty_as_default(deserializer).map(Self)
    }
}

/// An optional value in a position where the field helpers can't be used,
/// treating `[]`, `{}` and `null` as `None`.
#[derive(Debug, Clone, Default)]
pub struct PhpOption<T>(pub Option<T>);

impl<'de, T: DeserializeOwned> Deserialize<'de> for PhpOption<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        empty_as_none(deserializer).map(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Deserialize)]
    struct Holder {
        #[serde(default, deserialize_with = "empty_as_default")]
        map: HashMap<String, i64>,
        #[serde(default, deserialize_with = "empty_as_none")]
        opt: Option<HashMap<String, i64>>,
    }

    #[test]
    fn empty_array_maps_to_default() {
        let h: Holder = serde_json::from_str(r#"{"map": [], "opt": []}"#).unwrap();
        assert!(h.map.is_empty());
        assert!(h.opt.is_none());
    }

    #[test]
    fn object_is_deserialized() {
        let h: Holder = serde_json::from_str(r#"{"map": {"a": 1}, "opt": {"b": 2}}"#).unwrap();
        assert_eq!(h.map["a"], 1);
        assert_eq!(h.opt.unwrap()["b"], 2);
    }

    #[test]
    fn empty_object_is_none_at_top_level() {
        let o: PhpOption<HashMap<String, i64>> = serde_json::from_str("{}").unwrap();
        assert!(o.0.is_none());
        let m: PhpMap<i64> = serde_json::from_str("[]").unwrap();
        assert!(m.0.is_empty());
    }

    #[test]
    fn missing_and_null_map_to_default() {
        let h: Holder = serde_json::from_str(r#"{"opt": null}"#).unwrap();
        assert!(h.map.is_empty());
        assert!(h.opt.is_none());
    }
}
