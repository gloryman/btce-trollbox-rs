extern crate rustc_serialize;
use rustc_serialize::json::Json;

#[derive(RustcDecodable, RustcEncodable, Clone, Debug)]
pub enum TickType {
    BID,
    ASK
}

impl TickType {
    fn from_string(label: &str) -> TickType {
        if label == "ask" {
            return TickType::ASK
        } else {
            return TickType::BID
        }
    }
}

#[derive(RustcDecodable, RustcEncodable, Clone, Debug)]
pub struct Tick {
    pub amount: f64,
    pub price: f64,
    pub tid: u64,
    pub timestamp: u64,
    pub typ: TickType
}


impl Tick {
/*    fn new() -> Tick {
        let tick = Tick {
            amount   : 0.0,
            price    : 0.0,
            tid      : 0,
            timestamp: 0,
            typ      : TickType::ASK,
        };

        return tick;
    }
*/
    pub fn from_json(rec: &Json) -> Option<Tick> {
        let obj = rec.as_object().unwrap();

        let tick = Tick {
            amount   : obj.get("amount").unwrap().as_f64().unwrap(),
            price    : obj.get("price").unwrap().as_f64().unwrap(),
            tid      : obj.get("tid").unwrap().as_u64().unwrap(),
            timestamp: obj.get("timestamp").unwrap().as_u64().unwrap(),
            typ      : obj.get("type").unwrap()
                                        .as_string()
                                        .map(|x| TickType::from_string(x))
                                        .unwrap(),
        };

        return Some(tick);
    }
}

pub type TicksList = Vec<Tick>;
