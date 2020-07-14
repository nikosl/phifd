use actix::prelude::*;
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    messages::{HeartBeat, Monitor, PeerStatus, Status},
    peer::Peer,
    phi::{PhiAccrualFailureDetector, PhiAccrualFailureDetectorBuilder, State},
};

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
}

pub struct InventoryActor {
    my_id: Uuid,
    my_addr: String,
    inv: HashMap<Uuid, PeerMonitor>,
    fd: PhiAccrualFailureDetectorBuilder,
}

impl InventoryActor {
    pub fn new(my_id: Uuid, my_addr: String) -> Self {
        InventoryActor {
            my_id,
            my_addr,
            inv: HashMap::new(),
            fd: PhiAccrualFailureDetectorBuilder::new(),
        }
    }
}

impl Actor for InventoryActor {
    type Context = Context<Self>;
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
