mod actor;
mod messages;
mod peer;
mod phi;
use crate::actor::heartbeat::{UdpActor, UdpPacket};

use actix::prelude::*;
use dotenv::dotenv;
use futures_util::stream::StreamExt;
use std::io::Result;
use std::{collections::HashMap, net::SocketAddr};
use tokio::net::UdpSocket;
use tokio_util::udp::UdpFramed;

use actix::io::SinkWrite;

use actor::inventory::InventoryActor;
use messages::{HBCodec, HeartBeat};
use uuid::Uuid;

#[actix_rt::main]
async fn main() {
    dotenv().ok();

    let me = Uuid::new_v4();

    let inv = InventoryActor::new(me.clone(), String::from("127.0.0.1:0"));
    let inv_addr = inv.start();

    let addr: SocketAddr = "127.0.0.1:41031".parse().unwrap();
    let sock = UdpSocket::bind(&addr).await.unwrap();
    println!(
        "Started udp server on: 127.0.0.1:{:?}",
        sock.local_addr().unwrap().port()
    );
    let (sink, stream) = UdpFramed::new(sock, messages::HBCodec).split();
    UdpActor::create(|p_ctx| {
        p_ctx.add_stream(
            stream.filter_map(|item: Result<(HeartBeat, SocketAddr)>| async {
                item.map(|(data, sender)| UdpPacket(data, sender)).ok()
            }),
        );
        UdpActor {
            me,
            sink: SinkWrite::new(sink, p_ctx),
            inventory: inv_addr,
            monitored: HashMap::new(),
        }
    });

    println!("Running server on 127.0.0.1:12345");

    tokio::signal::ctrl_c().await.unwrap();
    println!("Ctrl-C received, shutting down");
    System::current().stop();
}
