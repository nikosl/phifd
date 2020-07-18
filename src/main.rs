#![allow(dead_code)]

mod actor;
mod handlers;
mod messages;
mod peer;
mod phi;

use crate::actor::heartbeat::{UdpActor, UdpPacket};

use actix::prelude::*;
use dotenv::dotenv;
use futures_util::stream::StreamExt;
use std::io::Result;
use std::{collections::HashMap, env, net::{SocketAddr, ToSocketAddrs}};
use tokio::net::UdpSocket;
use tokio_util::udp::UdpFramed;

use actix::io::SinkWrite;

use actix_files;
use actix_web::{web, App, HttpResponse, HttpServer};
use actor::{inventory::InventoryActor, monitor::MonitorActor};
use messages::HeartBeat;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    let m_cfg = env::var("PHI_ME").expect("PHI_ME is not set!");
    let raddr_cfg = env::var("PHI_REST").expect("PHI_REST is not set!");

    let me_info = peer::Peer::from(m_cfg);
    let me = me_info.id;

    let inv = InventoryActor::new(me);
    let inv_addr = inv.start();

    let addr: SocketAddr = me_info.address.as_str().to_socket_addrs()?.next().unwrap();
    let sock = UdpSocket::bind(&addr).await.unwrap();
    println!(
        "Started udp server on: {:?}",
        sock.local_addr().unwrap().port()
    );
    let (sink, stream) = UdpFramed::new(sock, messages::HBCodec).split();
    let hb = UdpActor::create(|p_ctx| {
        p_ctx.add_stream(
            stream.filter_map(|item: Result<(HeartBeat, SocketAddr)>| async {
                item.map(|(data, sender)| UdpPacket(data, sender)).ok()
            }),
        );
        UdpActor {
            me,
            sink: SinkWrite::new(sink, p_ctx),
            inventory: inv_addr.clone(),
            monitored: HashMap::new(),
        }
    });

    let monit = MonitorActor(std::collections::HashMap::new(), inv_addr.clone()).start();

    let state = handlers::AppState {
        inventory: inv_addr,
        heartbeat: hb,
        monit: monit,
        me: me_info,
    };

    HttpServer::new(move || {
        App::new()
            .data(state.clone())
            .service(
                web::scope("/api")
                    .route("/info", web::get().to(handlers::info))
                    .route("/register", web::post().to(handlers::register))
                    .route("/unregister/{id}", web::delete().to(handlers::unregister)),
            )
            // redirect to websocket.html
            .service(web::resource("/").route(web::get().to(|| {
                HttpResponse::Found()
                    .header("LOCATION", "/static/index.html")
                    .finish()
            })))
            // websocket
            .service(web::resource("/ws/").to(handlers::index))
            // static resources
            .service(actix_files::Files::new("/static/", "static/"))
    })
    .bind(raddr_cfg)?
    .run()
    .await
}
