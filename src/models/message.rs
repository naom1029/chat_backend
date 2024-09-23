use actix::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// クライアントから送信され送信されるメっセージ
#[derive(Clone, Message, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct ClientMessage {
    pub text: String,
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

// サーバーに参加するためのメッセージ
#[derive(Clone, Message)]
#[rtype(result = "Uuid")]
pub struct JoinServer {
    pub server_name: String,
    pub client_name: Option<String>,
    pub client: Recipient<ChatMessage>,
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
