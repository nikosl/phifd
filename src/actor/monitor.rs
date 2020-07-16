use super::inventory::InventoryActor;
use crate::messages;
use actix::prelude::*;
use actix::{Actor, Handler, StreamHandler};
use actix_web_actors::ws;
use dev::{MessageResponse, ResponseChannel};
use serde_json as json;
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};
use uuid::Uuid;
use ws::WebsocketContext;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Message)]
#[rtype(result = "ResponseId")]
pub struct Connect {
    pub addr: Recipient<messages::Status>,
}

pub struct ResponseId(uuid::Uuid);

impl<A, M> MessageResponse<A, M> for ResponseId
where
    A: Actor,
    M: Message<Result = ResponseId>,
{
    fn handle<R: ResponseChannel<M>>(self, _: &mut A::Context, tx: Option<R>) {
        if let Some(tx) = tx {
            tx.send(self);
        }
    }
}

/// Session is disconnected
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: Uuid,
}

pub struct MonitorActor(
    pub HashMap<Uuid, Recipient<messages::Status>>,
    pub Addr<InventoryActor>,
);

impl MonitorActor {
    fn notify_status(&self, msg: messages::Status) {
        for (_, addr) in self.0.iter() {
            let _ = addr.do_send(msg.clone());
        }
    }
}

impl Actor for MonitorActor {
    type Context = Context<Self>;
}

impl Handler<Connect> for MonitorActor {
    type Result = ResponseId;

    fn handle(&mut self, msg: Connect, ctx: &mut Context<Self>) -> Self::Result {
        println!("Someone joined");

        // register session with random id
        let id = ResponseId(uuid::Uuid::new_v4());
        self.0.insert(id.0, msg.addr);

        self.1
            .do_send(messages::StatusEvent::Subscribe(ctx.address()));
        id
    }
}

/// Handler for Disconnect message.
impl Handler<Disconnect> for MonitorActor {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        println!("Someone disconnected");

        self.0.remove(&msg.id);
        if self.0.is_empty() {
            self.1.do_send(messages::StatusEvent::UnSubscribe);
        }
    }
}

impl Handler<messages::Status> for MonitorActor {
    type Result = ();

    fn handle(&mut self, msg: messages::Status, _ctx: &mut Self::Context) {
        self.notify_status(msg);
    }
}

pub struct MonitorSession {
    pub id: Uuid,
    pub hb: Instant,
    pub monit: Addr<MonitorActor>,
}

impl MonitorSession {
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                println!("Websocket Client heartbeat failed, disconnecting!");

                act.monit.do_send(Disconnect { id: act.id });

                ctx.stop();

                return;
            }

            ctx.ping(b"");
        });
    }
}

impl Actor for MonitorSession {
    type Context = WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // we'll start heartbeat process on session start.
        self.hb(ctx);

        let addr = ctx.address();
        self.monit
            .send(Connect {
                addr: addr.recipient(),
            })
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(res) => act.id = res.0,
                    _ => ctx.stop(),
                }
                fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        // notify chat server
        self.monit.do_send(Disconnect { id: self.id });
        Running::Stop
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MonitorSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let msg = match msg {
            Err(_) => {
                ctx.stop();
                return;
            }
            Ok(msg) => msg,
        };

        match msg {
            ws::Message::Ping(msg) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            ws::Message::Pong(_) => {
                self.hb = Instant::now();
            }
            ws::Message::Text(_) => {}
            ws::Message::Binary(_) => {}
            ws::Message::Close(reason) => {
                ctx.close(reason);
                ctx.stop();
            }
            ws::Message::Continuation(_) => {
                ctx.stop();
            }
            ws::Message::Nop => (),
        }
    }
}

impl Handler<messages::Status> for MonitorSession {
    type Result = ();

    fn handle(&mut self, msg: messages::Status, ctx: &mut Self::Context) {
        ctx.text(json::to_string(&msg).unwrap());
    }
}
