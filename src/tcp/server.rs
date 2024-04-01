use actix::prelude::*;
use rand::{rngs::ThreadRng, Rng};
use std::collections::{HashMap, HashSet};

use super::session;
/*** Notes
    In the Actix framework for Rust, Handler is a trait defined to describe how to handle messages of specific types. An actor can implement Handler to handle certain types of messages.

    Specifically, Handler has two conditions:

    Self: Actor: This requires that any type implementing Handler must also be an actor, meaning it must implement the Actor trait.

    M: Message: This requires that the type M must be a message type that can be handled by the actor.

    Handler provides a general way to handle messages, streams, and futures. When an actor implements Handler, it needs to provide specific logic to handle that message. This is typically done by implementing one or more methods of the Handler trait.

    For example, if you have an actor that wants to handle messages of type MyMessage, you would implement Handler<MyMessage> for that actor and provide a handle() method with the specific logic to handle that message.

***/
/// Define a MessageHandler for a actor that wants to handle messages of type MyMessage, example: Messsage, Connect, Disconnect, Join in this case.

/// Send message to specific room
#[derive(Message)]
#[rtype(result = "()")]
pub struct Message {
    /// Id of the client session
    pub id: usize,
    /// Peer message
    pub msg: String,
    /// Room name
    pub room: String,
}

/// New chat session is created
#[derive(Message)]
#[rtype(usize)]
pub struct Connect {
    pub addr: Recipient<session::Message>,
}

/// Session is disconnected
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: usize,
}

/// Join room, if room does not exists create new one.
#[derive(Message)]
#[rtype(result = "()")]
pub struct Join {
    pub client_id: usize,

    pub room_name: String,
}

pub struct Server {
    sessions: HashMap<usize, Recipient<session::Message>>,
    rooms: HashMap<String, HashSet<usize>>,
    rng: ThreadRng,
}

impl Default for Server {
    fn default() -> Self {
        let mut rooms = HashMap::new();
        rooms.insert("main".to_owned(), HashSet::new());

        Server {
            sessions: HashMap::new(),
            rooms,
            rng: rand::thread_rng(),
        }
    }
}

impl Server {
    fn send_message(&self, room: &str, message: &str, skip_id: usize) {}
}

impl Actor for Server {
    type Context = Context<Self>;
}

impl Handler<Connect> for Server {
    type Result = usize;

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        log::info!("Someone joined");

        self.send_message("main", "Someone joined", 0);

        let id = self.rng.gen::<usize>();
        self.sessions.insert(id, msg.addr);
        // auto join session to main room
        self.rooms.get_mut("main").unwrap().insert(id);
        id
    }
}

impl Handler<Disconnect> for Server {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, ctx: &mut Self::Context) -> Self::Result {
        log::info!("Someone disconnected");
    }
}

/// Handler for Message message.
impl Handler<Message> for Server {
    type Result = ();

    fn handle(&mut self, msg: Message, _: &mut Context<Self>) {}
}

impl Handler<Join> for Server {
    type Result = ();
    fn handle(&mut self, msg: Join, ctx: &mut Self::Context) -> Self::Result {
        let Join {
            client_id,
            room_name,
        } = msg;

        let mut rooms = Vec::new();

        for (n, sessions) in &mut self.rooms {
            if sessions.remove(&client_id) {
                rooms.push(n.to_owned());
            }
        }

        for room in rooms {
            self.send_message(&room, "Someone disconnected", 0);
        }

        if self.rooms.get_mut(&room_name).is_none() {
            self.rooms.insert(room_name.clone(), HashSet::new());
        }

        self.send_message(&room_name, "Someone connected", client_id);
        self.rooms.get_mut(&room_name).unwrap().insert(client_id);
    }
}
