use serde::Deserialize;

use crate::entities::IntelPortal;

/// endpoint reponse root
#[derive(Clone, Debug, Deserialize)]
pub struct IntelResponse {
    /// "result" node
    pub result: IntelPortal,
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
