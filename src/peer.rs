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
        let prop: Vec<&str> = str_peer.splitn(3,"/").collect();
        Peer::new(uuid::Uuid::parse_str(prop[0]).unwrap(), prop[1].to_owned(), prop[2].to_owned())
    }
    
}
