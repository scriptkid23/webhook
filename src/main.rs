mod tcp;
//hello
use actix::prelude::*;
use actix_web::{
    middleware::Logger, post, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder,
};
use actix_web_actors::ws;

use log::info;

use tcp::{Connect, Server};

use crate::tcp::tcp_server;

struct WsChatSession {
    /// unique session id
    id: usize,

    /// joined room
    room: String,
    /// peer name
    name: Option<String>,
    /// Chat server
    addr: Addr<Server>,
}

impl Handler<tcp::SessionMessage> for WsChatSession {
    type Result = ();
    fn handle(&mut self, msg: tcp::SessionMessage, ctx: &mut Self::Context) -> Self::Result {
        ctx.text(msg.0)
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsChatSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let msg = match msg {
            Err(_) => {
                ctx.stop();
                return;
            }
            Ok(msg) => msg,
        };

        info!("Websocket message: {msg:?}");
    }
}

impl Actor for WsChatSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let addr = ctx.address();
        self.addr
            .send(Connect {
                addr: addr.recipient(),
            })
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(res) => act.id = res,
                    // something is wrong with chat server
                    _ => ctx.stop(),
                }
                fut::ready(())
            })
            .wait(ctx);
    }
}

async fn chat_route(
    req: HttpRequest,
    stream: web::Payload,
    srv: web::Data<Addr<Server>>,
) -> Result<impl Responder, Error> {
    ws::start(
        WsChatSession {
            id: 0,
            room: "main".to_owned(),
            name: None,
            addr: srv.get_ref().clone(),
        },
        &req,
        stream,
    )
}

#[post("/{token}")]
async fn push(path: web::Path<(String,)>) -> HttpResponse {
    let token = path.into_inner().0;

    info!("{}", token);

    HttpResponse::Ok().body(format!("User detail"))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let server = tcp::Server::default().start();

    let srv = server.clone();

    tcp_server("127.0.0.1:12345", srv);

    log::info!("Starting HTTP server at http://localhost:8080");

    HttpServer::new(move || {
        let logger = Logger::default();

        App::new()
            .app_data(web::Data::new(server.clone()))
            .wrap(logger)
            .service(push)
            .service(web::resource("/ws").to(chat_route))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
