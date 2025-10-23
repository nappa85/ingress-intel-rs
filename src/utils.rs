use serde::{Deserialize, Deserializer};

pub(crate) fn deserialize_coord<'de, D: Deserializer<'de>>(deserializer: D) -> Result<f64, D::Error> {
    let temp = i64::deserialize(deserializer)?;
    Ok(temp as f64 / 1000000_f64)
}

pub(crate) fn deserialize_coord_opt<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<f64>, D::Error> {
    let temp = Option::<i64>::deserialize(deserializer)?;
    Ok(temp.map(|i| i as f64 / 1000000_f64))
}
