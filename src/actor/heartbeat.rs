use actix::prelude::*;
use std::time::Duration;
use uuid::Uuid;

use actix::io::SinkWrite;

use futures::stream::SplitSink;
use std::net::{SocketAddr, ToSocketAddrs};
use tokio_util::udp::UdpFramed;

use super::inventory::InventoryActor;
use crate::{
    messages::{HBCodec, HeartBeat, Monitor},
    phi,
};

type SinkItem = (HeartBeat, SocketAddr);
type UdpSink = SplitSink<UdpFramed<HBCodec>, SinkItem>;

pub struct UdpActor {
    pub me: Uuid,
    pub sink: SinkWrite<SinkItem, UdpSink>,
    pub inventory: Addr<InventoryActor>,
    pub monitored: std::collections::HashMap<Uuid, Addr<HeartBeatActor>>,
}

impl Actor for UdpActor {
    type Context = Context<Self>;
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct StopHeartbeat;

#[derive(Message)]
#[rtype(result = "()")]
pub struct UdpPacket(pub HeartBeat, pub SocketAddr);

impl Handler<HeartBeat> for UdpActor {
    type Result = ();

    fn handle(&mut self, msg: HeartBeat, _: &mut Context<Self>) {
        if let HeartBeat::DoPing(id, ts, addr) = msg {
            self.sink.write((HeartBeat::Ping(id, ts), addr)).unwrap();
        }
    }
}

impl Handler<Monitor> for UdpActor {
    type Result = ();

    fn handle(&mut self, msg: Monitor, ctx: &mut Context<Self>) {
        match msg {
            Monitor::Register(ref peer) => {
                println!("Register peer {:?}", peer);
                if peer.id == self.me {
                    return;
                }

                self.inventory.do_send(msg.clone());
                let addr = HeartBeatActor::new(
                    peer.id,
                    peer.name.clone(),
                    peer.address.clone(),
                    ctx.address(),
                )
                .start();
                self.monitored.insert(peer.id, addr);
            }
            Monitor::UnRegister(ref uuid) => {
                println!("Unregister peer {:?}", uuid);
                self.monitored
                    .remove(uuid)
                    .and_then::<Addr<HeartBeatActor>, _>(|addr| {
                        addr.do_send(StopHeartbeat);
                        None
                    });
                self.inventory.do_send(msg);
            }
        };
    }
}

impl StreamHandler<UdpPacket> for UdpActor {
    fn handle(&mut self, rmsg: UdpPacket, _: &mut Context<Self>) {
        println!("Received: ({:?}, {:?})", rmsg.0, rmsg.1);
        match rmsg.0 {
            HeartBeat::Ping(_id, _ts) => self
                .sink
                .write((HeartBeat::Pong(self.me, phi::now()), rmsg.1))
                .unwrap(),
            HeartBeat::Pong(id, _ts) => self.inventory.do_send(HeartBeat::Pong(id, phi::now())),
            _ => println!("Received non match: ({:?}, {:?})", rmsg.0, rmsg.1),
        };
    }
}

impl actix::io::WriteHandler<std::io::Error> for UdpActor {}

pub struct HeartBeatActor {
    id: Uuid,
    name: String,
    address: String,
    socket_address: SocketAddr,
    pinger: Addr<UdpActor>,
}

impl Actor for HeartBeatActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.heartbeat(ctx);
    }
}

impl actix::Supervised for HeartBeatActor {}

impl Handler<StopHeartbeat> for HeartBeatActor {
    type Result = ();

    fn handle(&mut self, _msg: StopHeartbeat, ctx: &mut Context<Self>) {
        ctx.stop();
    }
}

impl HeartBeatActor {
    pub fn new(id: Uuid, name: String, address: String, pinger: Addr<UdpActor>) -> HeartBeatActor {
        HeartBeatActor {
            id,
            name,
            pinger,
            address: address.clone(),
            socket_address: address.as_str().to_socket_addrs().expect("Invalid forwarding address specified").next().unwrap(),
        }
    }

    fn heartbeat(&self, ctx: &mut Context<Self>) {
        ctx.run_interval(Duration::from_millis(150), |actor, _ctx| {
            actor.pinger.do_send(HeartBeat::DoPing(
                actor.id,
                phi::now(),
                actor.socket_address,
            ));
        });
    }
}
