use super::WebSocketServer;
use actix::prelude::*;

use std::{
    io, net,
    str::FromStr,
    time::{Duration, Instant},
};

/// Chat server sends this messages to session
#[derive(Message)]
#[rtype(result = "()")]
pub struct Message(pub String);

pub struct Session {
    /// unique session id
    id: usize,
    /// this is address of chat server
    addr: Addr<WebSocketServer>,
    /// Client must send ping at least once per 10 seconds, otherwise we drop
    /// connection.
    hb: Instant,
    /// joined room
    room: String,
}

impl Session {
    pub fn new() {}

    fn hb(&self, ctx: &mut Context<Self>) {}
}
impl Actor for Session {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {}
}
