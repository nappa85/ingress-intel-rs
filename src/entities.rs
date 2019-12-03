use std::collections::HashMap;

use serde::Deserialize;

use serde_json::value::Value;

#[derive(Debug, Deserialize)]
pub struct IntelResponse {
    pub result: IntelMap,
}

#[derive(Debug, Deserialize)]
pub struct IntelMap {
    pub map: HashMap<String, IntelResult>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]  
pub enum IntelResult {
    Error(IntelError),
    Entities(IntelEntities),
}

#[derive(Debug, Deserialize)]
pub struct IntelError {
    pub error: String,
}

#[derive(Debug, Deserialize)]
pub struct IntelEntities {
    #[serde(rename = "gameEntities")]
    pub entities: Vec<IntelEntity>
}

#[derive(Debug, Deserialize)]
pub struct IntelEntity(String, i64, Vec<Value>);

impl IntelEntity {
    pub fn get_id(&self) -> &str {
        &self.0
    }

    pub fn get_name(&self) -> Option<&str> {
        if self.2[0].as_str() == Some("p") {
            self.2[8].as_str()
        }
        else {
            None
        }
    }

    pub fn get_latitude(&self) -> Option<f64> {
        if self.2[0].as_str() == Some("p") {
            self.2[2].as_f64().map(|n| n / 1000000_f64)
        }
        else {
            None
        }
    }

    pub fn get_longitude(&self) -> Option<f64> {
        if self.2[0].as_str() == Some("p") {
            self.2[3].as_f64().map(|n| n / 1000000_f64)
        }
        else {
            None
        }
    }
}
