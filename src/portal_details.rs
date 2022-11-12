use serde::Deserialize;

use serde_json::value::Value;

/// endpoint reponse root
#[derive(Clone, Debug, Deserialize)]
pub struct IntelResponse {
    /// "result" node
    pub result: IntelPortal,
}

/// endpoint main entity
#[derive(Clone, Debug, Deserialize)]
pub struct IntelPortal(
    String,
    String,
    i64,
    i64,
    u8,
    f64,
    u8,
    String,
    String,
    Vec<Value>,
    bool,
    bool,
    Value,
    i64,
    Vec<Option<IntelMod>>,
    Vec<Option<IntelResonator>>,
    String,
    Vec<Value>,
);

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

    /// returns resonators state
    pub fn get_mods(&self) -> &[Option<IntelMod>] {
        &self.14
    }

    /// returns resonators state
    pub fn get_resonators(&self) -> &[Option<IntelResonator>] {
        &self.15
    }
}

/// portal deployed mod
#[derive(Clone, Debug, Deserialize)]
pub struct IntelMod(String, String, String, Value);

impl IntelMod {
    /// returns mod owner
    pub fn get_owner(&self) -> &str {
        &self.0
    }

    /// returns mod name/type
    pub fn get_name(&self) -> &str {
        &self.1
    }

    /// returns mod rarity
    pub fn get_rarity(&self) -> &str {
        &self.2
    }

    /// returns mod stats
    pub fn get_stats(&self) -> &Value {
        &self.3
    }
}

/// portal deployed resonator
#[derive(Clone, Debug, Deserialize)]
pub struct IntelResonator(String, u8, u16);

impl IntelResonator {
    /// returns resonator owner
    pub fn get_owner(&self) -> &str {
        &self.0
    }

    /// returns resonator level
    pub fn get_level(&self) -> u8 {
        self.1
    }

    /// returns resonator energy
    pub fn get_energy(&self) -> u16 {
        self.2
    }
}
