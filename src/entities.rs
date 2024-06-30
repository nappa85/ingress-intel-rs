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
/// struct is made of (id, timestamp, values)
/// values vary based on type
/// e.g. for portal
/// [
///     0: type,
///     1: faction,
///     2: latitude,
///     3: longitude,
///     4: level,
///     5: health,
///     6: resCount,
///     7: image,
///     8: title,
///     9: ornaments,
///     10: mission,
///     11: mission50plus,
///     12: artifactBrief,
///     13: timestamp,
///     14: mods,
///     15: resonators,
///     16: owner,
///     17: artifactDetail,
///     18: history
/// ]
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct IntelEntity(String, i64, Vec<Value>);

macro_rules! portal {
    ($this:ident, $index:literal: $name:ident, $var:ident => $code:block) => {
        if $this.is_portal() {
            if let Some($var) = $this.2.get($index) {
                $code
            } else {
                warn!("Portal without {}: {:?}", std::stringify!($name), $this);
                None
            }
        } else {
            None
        }
    };
}

impl IntelEntity {
    /// returns entity id
    pub fn get_id(&self) -> Option<&str> {
        if self.is_portal() {
            Some(&self.0)
        } else {
            None
        }
    }

    /// returns entity faction
    pub fn is_portal(&self) -> bool {
        self.2.first().and_then(Value::as_str) == Some("p")
    }

    /// returns faction if entity is a portal
    pub fn get_faction(&self) -> Option<Faction> {
        portal!(self, 1: faction, v => {
            match v.as_str() {
                Some("N") => Some(Faction::Neutral),
                Some("E") => Some(Faction::Enlightened),
                Some("R") => Some(Faction::Resistance),
                Some("M") => Some(Faction::Machina),
                _ => {
                    warn!("Unknown faction {:?}", self);
                    None
                }
            }
        })
    }

    /// returns latitude if entity is a portal
    pub fn get_latitude(&self) -> Option<f64> {
        portal!(self, 2: latitude, v => {
            Some(v.as_f64()? / 1000000_f64)
        })
    }

    /// returns longitude if entity is a portal
    pub fn get_longitude(&self) -> Option<f64> {
        portal!(self, 3: longitude, v => {
            Some(v.as_f64()? / 1000000_f64)
        })
    }

    /// returns level if entity is a portal
    pub fn get_level(&self) -> Option<u8> {
        portal!(self, 4: level, v => {
            Some(v.as_u64()? as u8)
        })
    }

    /// returns health if entity is a portal
    pub fn get_health(&self) -> Option<u8> {
        portal!(self, 5: health, v => {
            Some(v.as_u64()? as u8)
        })
    }

    /// returns resCount if entity is a portal
    pub fn get_res_count(&self) -> Option<u8> {
        portal!(self, 6: resCount, v => {
            Some(v.as_u64()? as u8)
        })
    }

    /// returns image if entity is a portal
    pub fn get_image(&self) -> Option<&str> {
        portal!(self, 7: image, v => {
            v.as_str()
        })
    }

    /// returns title if entity is a portal
    pub fn get_name(&self) -> Option<&str> {
        portal!(self, 8: title, v => {
            v.as_str()
        })
    }

    /// returns ornaments if entity is a portal
    pub fn get_ornaments(&self) -> Option<&Vec<Value>> {
        portal!(self, 9: ornaments, v => {
            v.as_array()
        })
    }

    /// returns mission if entity is a portal
    pub fn get_mission(&self) -> Option<bool> {
        portal!(self, 10: mission, v => {
            v.as_bool()
        })
    }

    /// returns mission50plus if entity is a portal
    pub fn get_mission50plus(&self) -> Option<bool> {
        portal!(self, 11: mission50plus, v => {
            v.as_bool()
        })
    }

    /// returns artifactBrief if entity is a portal
    pub fn get_artifact_brief(&self) -> Option<&Vec<Value>> {
        portal!(self, 12: artifactBrief, v => {
            v.as_array()
        })
    }

    /// returns timestamp if entity is a portal
    pub fn get_timestamp(&self) -> Option<u64> {
        portal!(self, 13: timestamp, v => {
            v.as_u64()
        })
    }

    /// returns mods if entity is a portal
    pub fn get_mods(&self) -> Option<&Vec<Value>> {
        portal!(self, 14: mods, v => {
            v.as_array()
        })
    }

    /// returns resonators if entity is a portal
    pub fn get_resonators(&self) -> Option<&Vec<Value>> {
        portal!(self, 15: resonators, v => {
            v.as_array()
        })
    }

    /// returns owner if entity is a portal
    pub fn get_owner(&self) -> Option<&str> {
        portal!(self, 16: owner, v => {
            v.as_str()
        })
    }

    /// returns artifactDetail if entity is a portal
    pub fn get_artifact_detail(&self) -> Option<&Vec<Value>> {
        portal!(self, 17: artifactDetail, v => {
            v.as_array()
        })
    }

    /// returns history if entity is a portal
    pub fn get_history(&self) -> Option<&Vec<Value>> {
        portal!(self, 18: history, v => {
            v.as_array()
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
/// Factions
pub enum Faction {
    /// Neutral
    Neutral,
    /// Enlightened
    Enlightened,
    /// Resistance
    Resistance,
    /// Machina
    Machina,
}

impl Faction {
    /// checks if neutral
    pub fn is_neutral(&self) -> bool {
        matches!(self, Faction::Neutral)
    }
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
