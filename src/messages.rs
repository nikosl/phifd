use crate::actor::monitor::MonitorActor;
use actix::prelude::{Addr, Message};

use serde_derive::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{peer::Peer, phi::State};

use bytes::{BufMut, BytesMut};
use serde_json as json;
use std::{io, net::SocketAddr};
use tokio_util::codec::{Decoder, Encoder};

#[derive(Debug, Clone, Serialize, Deserialize, Message)]
#[rtype(result = "()")]
pub enum HeartBeat {
    Ping(Uuid, u128),
    Pong(Uuid, u128),
    Unknown,
    DoPing(Uuid, u128, SocketAddr),
}

#[derive(Clone, Message)]
#[rtype(result = "()")]
pub enum StatusEvent {
    Subscribe(Addr<MonitorActor>),
    UnSubscribe,
}

#[derive(Debug, Clone, Serialize, Deserialize, Message)]
#[rtype(result = "()")]
pub struct PeerStatus {
    pub id: Uuid,
    pub name: String,
    pub address: String,
    pub history: std::vec::Vec<u128>,
    pub phi: f64,
    pub state: State,
    pub last: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize, Message)]
#[rtype(result = "()")]
pub struct Status(pub std::collections::HashMap<Uuid, PeerStatus>);

#[derive(Debug, Clone, Serialize, Deserialize, Message)]
#[rtype(result = "()")]
pub enum Monitor {
    Register(Peer),
    UnRegister(Uuid),
}

pub struct HBCodec;

impl Decoder for HBCodec {
    type Item = HeartBeat;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        Ok(json::from_slice::<HeartBeat>(&src)
            .ok()
            .or(Some(HeartBeat::Unknown)))
    }
}

impl Encoder<HeartBeat> for HBCodec {
    type Error = io::Error;

    fn encode(&mut self, msg: HeartBeat, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let msg = json::to_string(&msg).unwrap();
        let msg_ref: &[u8] = msg.as_ref();

        dst.reserve(msg_ref.len());
        dst.put(msg_ref);

        Ok(())
    }
}
