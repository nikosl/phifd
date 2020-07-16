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

impl From<String> for Peer {
    fn from(str_peer: String) -> Self {
        let prop: Vec<&str> = str_peer.splitn(2,":").collect();
        Peer{
            id: uuid::Uuid::new_v4(),
            name:  prop[0].to_owned(),
            address: prop[1].to_owned(),
        }
    }
    
}
