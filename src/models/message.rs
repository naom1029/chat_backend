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

#[derive(Clone, Message)]
#[rtype(result = "Uuid")]
pub struct JoinServer(pub String, pub Option<String>, pub Recipient<ServerMessage>);

#[derive(Clone, Message)]
#[rtype(result = "()")]
pub struct LeaveServer(pub String, pub Uuid);
#[derive(Clone, Message)]
#[rtype(result = "Vec<String>")]
pub struct ListServer;

#[derive(Clone, Message)]
#[rtype(result = "()")]
pub struct SendMessage(pub String, pub Uuid, pub String);
