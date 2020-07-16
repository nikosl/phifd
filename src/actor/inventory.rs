use actix::prelude::*;
use std::{time::Duration, collections::HashMap};
use uuid::Uuid;

use crate::{
    messages::{HeartBeat, Monitor, PeerStatus, Status, StatusEvent, self},
    phi::{PhiAccrualFailureDetector, PhiAccrualFailureDetectorBuilder, State, self},
};
use super::monitor::MonitorActor;

pub struct PeerMonitor {
    id: Uuid,
    name: String,
    address: String,
    status: PhiAccrualFailureDetector,
}

impl PeerMonitor {
    pub fn new(
        id: Uuid,
        name: String,
        address: String,
        status: PhiAccrualFailureDetector,
    ) -> PeerMonitor {
        PeerMonitor {
            id,
            name,
            address,
            status,
        }
    }

    pub fn heartbeat(&mut self, now: u128) {
        self.status.heartbeat(now);
    }

    pub fn state(&self, now: u128) -> State {
        self.status.state(now)
    }

    pub fn last(&self) -> u128 {
        self.status.last()
    }

    pub fn history(&self, num: usize) -> std::vec::Vec<u128> {
        self.status.history(num)
    }
}

impl From<&PeerMonitor> for PeerStatus {
    fn from(item: &PeerMonitor) -> Self {
        let st = item.state(phi::now());
        let phi  = match st {
            phi::State::Alive(p) => p,
            phi::State::Dead(p) => p,
        };
        PeerStatus {
            id: item.id.clone(),
            name: item.name.clone(),
            address: item.address.clone(),
            history: item.history(20),
            phi: phi,
            state: st,
            last: item.last(),
        }
    }
}

pub struct InventoryActor {
    my_id: Uuid,
    inv: HashMap<Uuid, PeerMonitor>,
    fd: PhiAccrualFailureDetectorBuilder,
    subs: bool,
    monit: Option<Addr<MonitorActor>>
}

impl InventoryActor {
    pub fn new(my_id: Uuid) -> Self {
        InventoryActor {
            my_id,
            inv: HashMap::new(),
            fd: PhiAccrualFailureDetectorBuilder::new(),
            subs: false,
            monit: None,
        }
    }

    fn get_status(&self)-> Status {
        let mut st = messages::Status(std::collections::HashMap::new());
        for (id, peer) in  self.inv.iter(){
            st.0.insert(id.clone(), PeerStatus::from(peer));
        }
        st
    }

    fn push_status(&self, ctx: &mut Context<Self>) {
        ctx.run_interval(Duration::new(1, 0), |actor, _ctx| {
            if actor.subs {
                if let Some(addr) = &actor.monit {
                    addr.do_send(actor.get_status());
                }
            }
        });
    }
}

impl Actor for InventoryActor {
    type Context = Context<Self>;
  
    fn started(&mut self, ctx: &mut Self::Context) {
        self.push_status(ctx);
    }
}

impl Handler<Monitor> for InventoryActor {
    type Result = ();

    fn handle(&mut self, msg: Monitor, _ctx: &mut Context<Self>) {
        match msg {
            Monitor::Register(peer) => {
                let monit = PeerMonitor::new(peer.id, peer.name, peer.address, self.fd.build());
                self.inv.insert(peer.id, monit);
            }
            Monitor::UnRegister(uuid) => {
                self.inv.remove(&uuid);
            }
        };
    }
}

impl Handler<HeartBeat> for InventoryActor {
    type Result = ();

    fn handle(&mut self, msg: HeartBeat, _ctx: &mut Context<Self>) {
        if let HeartBeat::Pong(id, ts) = msg {
            if let Some(p) = self.inv.get_mut(&id) {
                p.status.heartbeat(ts);
            }
        }
    }
}

impl Handler<StatusEvent> for InventoryActor {
    type Result = ();

    fn handle(&mut self, msg: StatusEvent, _ctx: &mut Context<Self>) {
        match msg {
            StatusEvent::Subscribe(addr) => {
                self.monit = Some(addr);
                self.subs = true;
            }
            StatusEvent::UnSubscribe => {}
        };
    }
}