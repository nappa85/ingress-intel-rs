use std::collections::HashMap;

use serde::Deserialize;

use serde_json::value::Value;

/// endpoint reponse root
#[derive(Debug, Deserialize)]
pub struct IntelResponse {
    /// "result" node
    pub result: IntelMap,
}

/// endpoint response "map" node
#[derive(Debug, Deserialize)]
pub struct IntelMap {
    /// "map" node
    pub map: HashMap<String, IntelResult>,
}

/// endpoint response "map" contents
#[derive(Debug, Deserialize)]
#[serde(untagged)]  
pub enum IntelResult {
    /// error entity
    Error(IntelError),
    /// ok entity
    Entities(IntelEntities),
}

/// endpoint error type
#[derive(Debug, Deserialize)]
pub struct IntelError {
    /// "error" node
    pub error: String,
}

/// endpoint ok type
#[derive(Debug, Deserialize)]
pub struct IntelEntities {
    /// "gameEntities" node
    #[serde(rename = "gameEntities")]
    pub entities: Vec<IntelEntity>
}

/// endpoint main entity
#[derive(Debug, Deserialize)]
pub struct IntelEntity(String, i64, Vec<Value>);

impl IntelEntity {
    /// returns entity id
    pub fn get_id(&self) -> &str {
        &self.0
    }

    /// returns name if entity is a portal
    pub fn get_name(&self) -> Option<&str> {
        if self.2[0].as_str() == Some("p") {
            self.2[8].as_str()
        }
        else {
            None
        }
    }

    /// returns latitude if entity is a portal
    pub fn get_latitude(&self) -> Option<f64> {
        if self.2[0].as_str() == Some("p") {
            self.2[2].as_f64().map(|n| n / 1000000_f64)
        }
        else {
            None
        }
    }

    /// returns longitude if entity is a portal
    pub fn get_longitude(&self) -> Option<f64> {
        if self.2[0].as_str() == Some("p") {
            self.2[3].as_f64().map(|n| n / 1000000_f64)
        }
        else {
            None
        }
    }
}
