use serde::Deserialize;

use serde_json::value::Value;
use tracing::warn;

use crate::entities::Faction;

/// endpoint reponse root
#[derive(Clone, Debug, Deserialize)]
pub struct IntelResponse {
    /// "result" node
    pub result: IntelPortal,
}

/// endpoint main entity
#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize)]
pub struct IntelPortal(
    /// 0: type,
    String,
    /// 1: faction,
    String,
    /// 2: latitude,
    i64,
    /// 3: longitude,
    i64,
    /// 4: level,
    u8,
    /// 5: health,
    u8,
    /// 6: resCount,
    u8,
    /// 7: image,
    Option<String>,
    /// 8: title,
    String,
    /// 9: ornaments,
    Vec<Value>,
    /// 10: mission,
    bool,
    /// 11: mission50plus,
    bool,
    /// 12: artifactBrief,
    Value,
    /// 13: timestamp,
    i64,
    /// 14: mods,
    Vec<Option<IntelMod>>,
    /// 15: resonators,
    Vec<Option<IntelResonator>>,
    /// 16: owner,
    Option<String>,
    /// 17: artifactDetail,
    Vec<Value>,
    /// 18: history
    #[serde(default)]
    Value,
);

impl IntelPortal {
    /// returns portal faction
    pub fn get_faction(&self) -> Option<Faction> {
        match self.1.as_str() {
            "N" => Some(Faction::Neutral),
            "E" => Some(Faction::Enlightened),
            "R" => Some(Faction::Resistance),
            "M" => Some(Faction::Machina),
            _ => {
                warn!("Unknown faction {}", self.1);
                None
            }
        }
    }

    /// returns portal latitude
    pub fn get_latitude(&self) -> f64 {
        (self.2 as f64) / 1000000_f64
    }

    /// returns portal longitude
    pub fn get_longitude(&self) -> f64 {
        (self.3 as f64) / 1000000_f64
    }

    /// returns portal level
    pub fn get_level(&self) -> u8 {
        self.4
    }

    /// returns portal health
    pub fn get_health(&self) -> u8 {
        self.5
    }

    /// returns portal resCount
    pub fn get_res_count(&self) -> u8 {
        self.6
    }

    /// returns portal image url
    pub fn get_image(&self) -> Option<&str> {
        self.7.as_deref()
    }

    /// returns portal title
    pub fn get_title(&self) -> &str {
        &self.8
    }

    /// returns resonators state
    pub fn get_mods(&self) -> &[Option<IntelMod>] {
        &self.14
    }

    /// returns resonators state
    pub fn get_resonators(&self) -> &[Option<IntelResonator>] {
        &self.15
    }

    /// returns portal owner
    pub fn get_owner(&self) -> Option<&str> {
        self.16.as_deref()
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

#[cfg(test)]
mod tests {
    #[test]
    fn owned_portal() {
        let s = r#"{"result":["p","R",45599806,12377142,1,85,1,"https://lh3.googleusercontent.com/ht0FYXJzAnMG_yhfC7gxefVtrJ3zW4LGifs7Ek_4_JORVzQ4DovLSQ3RpRnunQYOTOmE_LOrWVmRSRm256BR0ivO_Ns","S. Cipriano - Cimitero",["sc5_p"],false,false,null,1720246737675,[null,null,null,null],[["TerminateThis",5,2550]],"TerminateThis",["","",[]],3]}"#;
        let jd = &mut serde_json::Deserializer::from_str(s);
        serde_path_to_error::deserialize::<_, super::IntelResponse>(jd).unwrap();
    }

    #[test]
    fn white_portal() {
        let s = r#"{"result":["p","N",45599078,12341800,1,0,0,"https://lh3.googleusercontent.com/0Mc7PhcBGSl0oUwplGnFOMGox2mLahSL_8K69AFAXhXWHknDfuDqXqGwXhriF6AJv9UV03RlqXNnLJpRQ862geFPOg","Casale sul Sile- Capitello Mariano sulla Restera",["sc5_p"],false,false,null,1711348025166,[null,null,null,null],[],"",["","",[]]]}"#;
        let jd = &mut serde_json::Deserializer::from_str(s);
        serde_path_to_error::deserialize::<_, super::IntelResponse>(jd).unwrap();
    }

    #[test]
    fn machina_portal() {
        let s = r#"{"result":["p","M",45590126,12338500,5,1,8,"https://lh3.googleusercontent.com/jxuZSfHc52kZnwWbz9a9FxFMKjVTenxSWIBeRqH8DVSHtrNig8gam7a9uxOk-tYFMMSJE2RQTZZEFfpFdsAmYt-oThu0","Fontana Ottagonale",["sc5_p"],false,false,null,1720200758276,[["__MACHINA__","SoftBank Ultra Link","VERY_RARE",{"LINK_DEFENSE_BOOST":"1500","OUTGOING_LINKS_BONUS":"8","LINK_RANGE_MULTIPLIER":"5000","REMOVAL_STICKINESS":"150000"}],["__MACHINA__","SoftBank Ultra Link","VERY_RARE",{"LINK_DEFENSE_BOOST":"1500","OUTGOING_LINKS_BONUS":"8","LINK_RANGE_MULTIPLIER":"5000","REMOVAL_STICKINESS":"150000"}],["__MACHINA__","SoftBank Ultra Link","VERY_RARE",{"LINK_DEFENSE_BOOST":"1500","OUTGOING_LINKS_BONUS":"8","LINK_RANGE_MULTIPLIER":"5000","REMOVAL_STICKINESS":"150000"}],["__MACHINA__","SoftBank Ultra Link","VERY_RARE",{"LINK_DEFENSE_BOOST":"1500","OUTGOING_LINKS_BONUS":"8","LINK_RANGE_MULTIPLIER":"5000","REMOVAL_STICKINESS":"150000"}]],[["__MACHINA__",5,30],["__MACHINA__",5,30],["__MACHINA__",5,30],["__MACHINA__",5,30],["__MACHINA__",5,30],["__MACHINA__",5,30],["__MACHINA__",5,30],["__MACHINA__",5,30]],"_\u0336\u0331\u030d_\u0334\u0333\u0349\u0306\u0308\u0301M\u0337\u0354\u0324\u0352\u0104\u0337\u030dC\u0334\u033c\u0315\u0345H\u0336\u0339\u0355\u033c\u033e\u1e2c\u0335\u0307\u033e\u0313N\u0335\u033a\u0355\u0352\u0300\u030d\u00c4\u0334\u031e\u0330\u0301_\u0334\u0326\u0300\u0346\u0313_\u0337\u0323\u0308\u0301",["","",[]],3]}"#;
        let jd = &mut serde_json::Deserializer::from_str(s);
        serde_path_to_error::deserialize::<_, super::IntelResponse>(jd).unwrap();
    }
}
