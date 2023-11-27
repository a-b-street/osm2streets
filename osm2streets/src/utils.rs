// Copied from https://github.com/a-b-street/abstreet/tree/main/abstutil/src to reduce dependencies

use std::collections::BTreeMap;

use anyhow::Result;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Format a number with commas
pub fn prettyprint_usize(x: usize) -> String {
    let num = format!("{}", x);
    let mut result = String::new();
    let mut i = num.len();
    for c in num.chars() {
        result.push(c);
        i -= 1;
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
    }
    result
}

/// Serializes a `usize` as a `u32` to save space. Useful when you need `usize` for indexing, but
/// the values don't exceed 2^32.
pub fn serialize_usize<S: Serializer>(x: &usize, s: S) -> Result<S::Ok, S::Error> {
    if let Ok(x) = u32::try_from(*x) {
        x.serialize(s)
    } else {
        Err(serde::ser::Error::custom(format!("{} can't fit in u32", x)))
    }
}

/// Deserializes a `usize` from a `u32`.
pub fn deserialize_usize<'de, D: Deserializer<'de>>(d: D) -> Result<usize, D::Error> {
    let x = <u32>::deserialize(d)?;
    Ok(x as usize)
}

/// Serializes a BTreeMap as a list of tuples. Necessary when the keys are structs; see
/// https://github.com/serde-rs/json/issues/402.
pub fn serialize_btreemap<S: Serializer, K: Serialize, V: Serialize>(
    map: &BTreeMap<K, V>,
    s: S,
) -> Result<S::Ok, S::Error> {
    map.iter().collect::<Vec<(_, _)>>().serialize(s)
}

/// Deserializes a BTreeMap from a list of tuples. Necessary when the keys are structs; see
/// https://github.com/serde-rs/json/issues/402.
pub fn deserialize_btreemap<
    'de,
    D: Deserializer<'de>,
    K: Deserialize<'de> + Ord,
    V: Deserialize<'de>,
>(
    d: D,
) -> Result<BTreeMap<K, V>, D::Error> {
    let vec = <Vec<(K, V)>>::deserialize(d)?;
    let mut map = BTreeMap::new();
    for (k, v) in vec {
        map.insert(k, v);
    }
    Ok(map)
}
