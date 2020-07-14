use serde_derive::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Peer {
    pub id: Uuid,
    pub name: String,
    pub address: String,
}

impl Peer {
    pub fn new(id: Uuid, name: String, address: String) -> Peer {
        Peer { id, name, address }
    }
}
