use actix::prelude::*;
use actix_web::dev::Server;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// クライアントから送信されるメッセージ
#[derive(Clone, Message, Serialize, Deserialize)]
#[rtype(result = "()")]
#[serde(tag = "type")]
pub enum ClientMessage {
    Command(CommandMessage),
    Chat(ClientChatMessage),
}

#[derive(Clone, Message, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct ClientChatMessage {
    pub text: String,
}
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct CommandMessage {
    pub command: String,
    pub args: Option<String>, // コマンドに引数がある場合
}

// チャットメッセージ
#[derive(Clone, Message, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct ChatMessage {
    pub id: String,
    pub text: String,
    pub timestamp: String,
}

// システムメッセージ
#[derive(Clone, Message, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct SystemMessage {
    pub text: String,
}
#[derive(Clone, Message, Serialize, Deserialize)]
#[rtype(result = "()")]
#[serde(tag = "type")]
pub enum ServerMessage {
    Chat(ChatMessage),
    System(SystemMessage),
    List { rooms: Vec<String> },
}

// サーバーに参加するためのメッセージ
#[derive(Clone, Message)]
#[rtype(result = "Uuid")]
pub struct JoinServer {
    pub server_name: String,
    pub client_name: Option<String>,
    pub client: Recipient<ServerMessage>,
}

// サーバーから退出するためのメッセージ
#[derive(Clone, Message, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct LeaveServer {
    pub server_name: String,
    pub client_id: Uuid,
}

// サーバーのリストを取得するためのメッセージ
#[derive(Clone, Message, Serialize, Deserialize)]
#[rtype(result = "Vec<String>")]
pub struct ListServer;

// サーバーを選択するためのメッセージ
pub struct SelectServer {
    pub server_name: String,
    pub client_id: Uuid,
}

#[derive(Clone, Message, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct SendMessage(pub String, pub Uuid, pub String);
