use super::{
    codec::{ChatCodec, ChatRequest, ChatResponse},
    server, Server,
};
use actix::{prelude::*, spawn};

use tokio::{
    io::{split, WriteHalf},
    net::{TcpListener, TcpStream},
};
use tokio_util::codec::FramedRead;

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
    addr: Addr<Server>,
    /// Client must send ping at least once per 10 seconds, otherwise we drop
    /// connection.
    hb: Instant,
    /// joined room
    room: String,

    framed: actix::io::FramedWrite<ChatResponse, WriteHalf<TcpStream>, ChatCodec>,
}

impl Session {
    pub fn new(
        addr: Addr<Server>,
        framed: actix::io::FramedWrite<ChatResponse, WriteHalf<TcpStream>, ChatCodec>,
    ) -> Session {
        Session {
            id: 0,
            addr,
            hb: Instant::now(),
            room: "main".to_owned(),
            framed,
        }
    }

    fn hb(&self, ctx: &mut Context<Self>) {
        ctx.run_interval(Duration::new(1, 0), |act, ctx| {
            if Instant::now().duration_since(act.hb) > Duration::new(10, 0) {
                log::error!("Client heartbeat failed, disconnecting!");
                // notify chat server
                todo!();
                // stop actor
                ctx.stop();
            }
            act.framed.write(ChatResponse::Ping)
        });
    }
}
impl Actor for Session {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);

        let addr = ctx.address();

        self.addr
            .send(server::Connect {
                addr: addr.recipient(),
            })
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(res) => act.id = res,
                    // something is wrong with chat server
                    _ => ctx.stop(),
                }
                actix::fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
        Running::Stop
    }
}

impl Handler<Message> for Session {
    type Result = ();

    fn handle(&mut self, msg: Message, _: &mut Context<Self>) -> Self::Result {
        self.framed.write(ChatResponse::Message(msg.0))
    }
}

impl actix::io::WriteHandler<io::Error> for Session {}

impl StreamHandler<Result<ChatRequest, io::Error>> for Session {
    fn handle(&mut self, item: Result<ChatRequest, io::Error>, ctx: &mut Self::Context) {
        todo!()
    }
}
pub fn tcp_server(_s: &str, server: Addr<Server>) {
    let addr = net::SocketAddr::from_str("0.0.0.0:12345").unwrap();
    spawn(async move {
        let listener = TcpListener::bind(&addr).await.unwrap();
        while let Ok((stream, _)) = listener.accept().await {
            let server = server.clone();
            Session::create(|ctx| {
                let (r, w) = split(stream);

                Session::add_stream(FramedRead::new(r, ChatCodec), ctx);
                Session::new(server, actix::io::FramedWrite::new(w, ChatCodec, ctx))
            });
        }
    });
}
