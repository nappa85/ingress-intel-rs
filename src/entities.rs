use std::collections::HashMap;

use serde::Deserialize;

use serde_json::value::Value;

use tracing::warn;

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
    pub entities: Vec<IntelEntity>,
}

/// endpoint main entity
#[derive(Debug, Deserialize)]
pub struct IntelEntity(String, i64, Vec<Value>);

impl IntelEntity {
    /// returns entity id
    pub fn is_portal(&self) -> bool {
        self.2.get(0).and_then(Value::as_str) == Some("p")
    }

    /// returns entity id
    pub fn get_id(&self) -> Option<&str> {
        if self.is_portal() {
            Some(&self.0)
        } else {
            None
        }
    }

    /// returns name if entity is a portal
    pub fn get_name(&self) -> Option<&str> {
        if self.is_portal() {
            if let Some(v) = self.2.get(8) {
                return v.as_str();
            } else {
                warn!("Portal without name: {:?}", self);
            }
        }
        None
    }

    /// returns latitude if entity is a portal
    pub fn get_latitude(&self) -> Option<f64> {
        if self.is_portal() {
            if let Some(v) = self.2.get(2) {
                Some(v.as_f64()? / 1000000_f64)
            } else {
                warn!("Portal without latitude: {:?}", self);
                None
            }
        } else {
            None
        }
    }

    /// returns longitude if entity is a portal
    pub fn get_longitude(&self) -> Option<f64> {
        if self.is_portal() {
            if let Some(v) = self.2.get(3) {
                Some(v.as_f64()? / 1000000_f64)
            } else {
                warn!("Portal without longitude: {:?}", self);
                None
            }
        } else {
            None
        }
    }

    /// returns faction if entity is a portal
    pub fn get_faction(&self) -> Option<Faction> {
        if let Some(v) = self.2.get(1) {
            match v.as_str() {
                Some("E") => Some(Faction::Enlightened),
                Some("R") => Some(Faction::Resistance),
                Some("M") => Some(Faction::Machina),
                _ => {
                    warn!("Unknown faction {:?}", self);
                    None
                }
            }
        } else {
            warn!("Entity without faction: {:?}", self);
            None
        }
    }

    /// returns level if entity is a portal
    pub fn get_level(&self) -> Option<u8> {
        if self.is_portal() {
            if let Some(v) = self.2.get(4) {
                return Some(v.as_u64()? as u8);
            } else {
                warn!("Portal without level: {:?}", self);
            }
        }
        None
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
/// Factions
pub enum Faction {
    /// Enlightened
    Enlightened,
    /// Resistance
    Resistance,
    /// Machina
    Machina,
}

impl Faction {
    /// checks if enlightened
    pub fn is_enlightened(&self) -> bool {
        matches!(self, Faction::Enlightened)
    }
    /// checks if resistance
    pub fn is_resistance(&self) -> bool {
        matches!(self, Faction::Resistance)
    }
    /// checks if machina
    pub fn is_machina(&self) -> bool {
        matches!(self, Faction::Machina)
    }
}
