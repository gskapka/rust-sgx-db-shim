use std::sync::Mutex;
use crate::types::Bytes;
use std::collections::HashMap;

lazy_static! {
    pub static ref DATABASE: Mutex<HashMap<Bytes, Bytes>> = {
        let db = HashMap::new();
        Mutex::new(db)
    };
}
