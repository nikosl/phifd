use crate::{actor, messages, peer};
use actix::Addr;
use actix_web::{error, web, Error, HttpRequest, HttpResponse, Responder};
use actix_web_actors::ws;
use actor::{heartbeat::UdpActor, inventory::InventoryActor, monitor::MonitorActor};
use futures::future::{ready, Ready};
use peer::Peer;
use std::time::Instant;

#[derive(Clone)]
pub struct AppState {
    pub inventory: Addr<InventoryActor>,
    pub heartbeat: Addr<UdpActor>,
    pub monit: Addr<MonitorActor>,
    pub me: Peer,
}

impl Responder for peer::Peer {
    type Error = Error;
    type Future = Ready<Result<HttpResponse, Error>>;

    fn respond_to(self, _req: &HttpRequest) -> Self::Future {
        let body = serde_json::to_string(&self).unwrap();

        // Create response and set content type
        ready(Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(body)))
    }
}

pub async fn register(peer: web::Json<peer::Peer>, data: web::Data<AppState>) -> HttpResponse {
    let addr = data.get_ref().heartbeat.clone();

    let res = addr
        .send(messages::Monitor::Register(peer.into_inner()))
        .await;
    match res {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn unregister(id: web::Path<uuid::Uuid>, data: web::Data<AppState>) -> HttpResponse {
    let addr = data.get_ref().heartbeat.clone();

    let res = addr
        .send(messages::Monitor::UnRegister(id.into_inner()))
        .await;
    match res {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn info(data: web::Data<AppState>) -> impl Responder {
    data.get_ref().me.clone()
}

pub async fn index(
    req: HttpRequest,
    stream: web::Payload,
    srv: web::Data<AppState>,
) -> Result<HttpResponse, error::Error> {
    ws::start(
        actor::monitor::MonitorSession {
            id: uuid::Uuid::new_v4(),
            hb: Instant::now(),
            monit: srv.get_ref().monit.clone(),
        },
        &req,
        stream,
    )
}
