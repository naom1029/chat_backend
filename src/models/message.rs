use actix::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Message, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct ClientMessage {
    pub text: String,
}
#[derive(Clone, Message, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct ServerMessage {
    pub id: String,
    pub text: String,
    pub timestamp: String,
}
#[derive(Clone, Message, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct SystemMessage {
    pub text: String,
}
#[derive(Clone, Message)]
#[rtype(result = "Uuid")]
pub struct JoinServer {
    pub server_name: String,
    pub client_name: Option<String>,
    pub client: Recipient<ServerMessage>,
}
#[derive(Clone, Message, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct LeaveServer {
    pub server_name: String,
    pub client_id: Uuid,
}
#[derive(Clone, Message, Serialize, Deserialize)]
#[rtype(result = "Vec<String>")]
pub struct ListServer;

pub struct SelectServer {
    pub server_name: String,
    pub client_id: Uuid,
}

#[derive(Clone, Message, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct SendMessage(pub String, pub Uuid, pub String);
