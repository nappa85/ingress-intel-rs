
use serde::Deserialize;

use serde_json::value::Value;

/// endpoint reponse root
#[derive(Debug, Deserialize)]
pub struct IntelResponse {
    /// "result" node
    pub result: IntelPortal,
}

/// endpoint main entity
#[derive(Debug, Deserialize)]
pub struct IntelPortal(String, String, i64, i64, u8, f64, u8, String, String, Vec<Value>, bool, bool, Value, i64, Vec<Option<IntelMod>>, Vec<Option<IntelResonator>>, String, Vec<Value>);

impl IntelPortal {
    /// returns portal name
    pub fn get_name(&self) -> &str {
        &self.8
    }

    /// returns portal image url
    pub fn get_url(&self) -> &str {
        &self.7
    }

    /// returns portal latitude
    pub fn get_latitude(&self) -> f64 {
        (self.2 as f64) / 1000000_f64
    }

    /// returns portal longitude
    pub fn get_longitude(&self) -> f64 {
        (self.3 as f64) / 1000000_f64
    }
}

/// portal deployed mod
#[derive(Debug, Deserialize)]
pub struct IntelMod(String, String, String, Value);

/// portal deployed resonator
#[derive(Debug, Deserialize)]
pub struct IntelResonator(String, u8, u16);
