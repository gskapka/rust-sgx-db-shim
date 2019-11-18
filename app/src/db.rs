use crate::types::Bytes;
use std::collections::HashMap;

lazy_static! {
    pub static ref DATABASE: HashMap<Bytes, Bytes> = {
        let mut db = HashMap::new();
        db
    };
}
