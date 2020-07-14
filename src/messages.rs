use crate::actor::{heartbeat::UdpActor, inventory::InventoryActor};
use actix::prelude::{Addr, Message};

use serde_derive::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{peer::Peer, phi::State};
use std::{io, net::SocketAddr};
use byteorder::{BigEndian, ByteOrder};
use bytes::{Buf, BufMut, BytesMut};
use serde_json as json;
use tokio_util::codec::{Decoder, Encoder};

#[derive(Clone)]
pub struct AppState {
    pub heartbeat: Addr<UdpActor>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Message)]
#[rtype(result = "()")]
pub enum HeartBeat {
    Ping(Uuid, u128),
    Pong(Uuid, u128),
    DoPing(Uuid, u128, SocketAddr),
}

#[derive(Debug, Clone, Serialize, Deserialize, Message)]
#[rtype(result = "()")]
pub struct PeerStatus {
    id: Uuid,
    name: String,
    address: String,
    history: std::vec::Vec<u128>,
    phi: f64,
    state: State,
    last: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize, Message)]
#[rtype(result = "()")]
pub struct Status(std::collections::HashMap<Uuid, PeerStatus>);

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
        let size = {
            if src.len() < 2 {
                return Ok(None);
            }
            BigEndian::read_u16(src.as_ref()) as usize
        };

        if src.len() >= size + 2 {
            src.advance(2);
            let buf = src.split_to(size);
            Ok(Some(json::from_slice::<HeartBeat>(&buf)?))
        } else {
            Ok(None)
        }
    }
}

impl Encoder<HeartBeat> for HBCodec {
    type Error = io::Error;

    fn encode(
        &mut self,
        msg: HeartBeat,
        dst: &mut BytesMut,
    ) -> Result<(), Self::Error> {
        let msg = json::to_string(&msg).unwrap();
        let msg_ref: &[u8] = msg.as_ref();

        dst.reserve(msg_ref.len() + 2);
        dst.put_u16(msg_ref.len() as u16);
        dst.put(msg_ref);

        Ok(())
    }
}