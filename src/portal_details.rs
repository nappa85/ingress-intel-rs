
use serde::Deserialize;

use serde_json::value::Value;

#[derive(Debug, Deserialize)]
pub struct IntelResponse {
    pub result: IntelPortal,
}

#[derive(Debug, Deserialize)]
pub struct IntelPortal(String, String, i64, i64, u8, f64, u8, String, String, Vec<Value>, bool, bool, Value, i64, Vec<IntelMod>, Vec<IntelResonator>, String, Vec<Value>);

impl IntelPortal {
    pub fn get_name(&self) -> &str {
        &self.8
    }

    pub fn get_url(&self) -> &str {
        &self.7
    }

    pub fn get_latitude(&self) -> f64 {
        (self.2 as f64) / 1000000_f64
    }

    pub fn get_longitude(&self) -> f64 {
        (self.3 as f64) / 1000000_f64
    }
}

#[derive(Debug, Deserialize)]
pub struct IntelMod(String, String, String, Value);

#[derive(Debug, Deserialize)]
pub struct IntelResonator(String, u8, u16);
