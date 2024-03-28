use std::str::from_utf8;
mod tcp;

use actix::{Actor, StreamHandler};
use actix_web::{
    get, middleware::Logger, post, web, App, Error, HttpRequest, HttpResponse, HttpServer,
    Responder,
};
use actix_web_actors::ws;

use log::info;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::tcp::tcp_server;

struct MyWs;

impl Actor for MyWs {
    type Context = ws::WebsocketContext<Self>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomMessage {
    pub event: String,
    pub payload: serde_json::Value,
}

/// Handler for ws::Message message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWs {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Text(text)) => {
                let msg_bytes = text.clone().into_bytes();
                match from_utf8(&msg_bytes) {
                    Ok(s) => {
                        println!("Result: {}", json!(s));
                    }
                    Err(e) => {
                        println!("Error: {:?}", e);
                    }
                }
                ctx.text(text)
            }

            _ => (),
        }
    }
}

async fn index(req: HttpRequest, payload: web::Payload) -> Result<HttpResponse, Error> {
    let resp = ws::start(MyWs {}, &req, payload);
    resp
}

#[post("/{token}")]
async fn push(path: web::Path<(String,)>) -> HttpResponse {
    let token = path.into_inner().0;

    info!("{}", token);

    HttpResponse::Ok().body(format!("User detail"))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "info");
    std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

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
            .route("/ws/", web::get().to(index))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
