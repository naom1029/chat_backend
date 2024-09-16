use actix::prelude::*;
use uuid::Uuid;

#[derive(Clone, Message)]
#[rtype(result = "()")]
pub struct ChatMessage(pub String);

#[derive(Clone, Message)]
#[rtype(result = "Uuid")]
pub struct JoinServer(pub String, pub Option<String>, pub Recipient<ChatMessage>);

#[derive(Clone, Message)]
#[rtype(result = "()")]
pub struct LeaveServer(pub String, pub Uuid);
#[derive(Clone, Message)]
#[rtype(result = "Vec<String>")]
pub struct ListServer;

#[derive(Clone, Message)]
#[rtype(result = "()")]
pub struct SendMessage(pub String, pub Uuid, pub String);
