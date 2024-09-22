use crate::models::user::User;
use crate::models::{
    ClientMessage, JoinServer, LeaveServer, ListServer, SendMessage, ServerMessage,
};
use actix::prelude::*;
use actix_broker::BrokerSubscribe;
use actix_web::dev::Server;
use actix_web::{
    web::{self, Data},
    Error, HttpRequest, HttpResponse,
};
use actix_web_actors::ws::{self, WebsocketContext};
use chrono::Utc;
use std::sync::atomic::AtomicUsize;
use std::{collections::HashMap, sync::Arc};

use log::{error, info, warn};
use tokio::sync::RwLock;
use uuid::{timestamp, Uuid};
#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct Message(pub String);

// ユーザーリストの型定義
type Users = Arc<RwLock<HashMap<Uuid, HashMap<usize, (User, Addr<WsChatServer>)>>>>;
// 接続IDを生成するためのカウンター
static NEXT_CONN_ID: AtomicUsize = AtomicUsize::new(1);

type Client = Recipient<ServerMessage>; // 送信先
type ClientConnections = HashMap<Uuid, Client>; // クライアントリスト
#[derive(Default)]
pub struct WsChatServer {
    // サーバーごとのクライアントリスト
    server_client_connections: HashMap<String, ClientConnections>,
}

impl WsChatServer {
    fn take_server(&mut self, server_name: &str) -> Option<ClientConnections> {
        let server = self.server_client_connections.get_mut(server_name)?;
        let server = std::mem::take(server);
        Some(server)
    }

    fn add_client_to_server(
        &mut self,
        server_name: &str,
        id: Option<Uuid>,
        client: Client,
    ) -> Uuid {
        let mut id = id.unwrap_or_else(Uuid::new_v4);

        if let Some(server) = self.server_client_connections.get_mut(server_name) {
            loop {
                if server.contains_key(&id) {
                    id = Uuid::new_v4();
                } else {
                    break;
                }
            }
            server.insert(id, client);
            return id;
        }
        // Create a new server for the first client
        let mut server: ClientConnections = HashMap::new();

        server.insert(id, client);
        self.server_client_connections
            .insert(server_name.to_owned(), server);

        id
    }
    fn send_chat_message(&mut self, server_name: &str, msg: &str, _src: Uuid) -> Option<()> {
        let clients = self.server_client_connections.get_mut(server_name)?;
        let message = ServerMessage {
            id: Uuid::new_v4().to_string(),
            text: msg.to_owned(),
            timestamp: Utc::now().to_rfc2822(),
        };
        for (id, client) in clients.iter() {
            match client.try_send(message.clone()) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Failed to send message to client {}: {}", id, e);
                }
            }
        }
        Some(())
    }
    fn send_system_message(&mut self, server_name: &str, msg: &str, client: Client) -> Option<()> {
        let message = ServerMessage {
            id: Uuid::new_v4().to_string(),
            text: msg.to_owned(),
            timestamp: Utc::now().to_rfc2822(),
        };
        match client.try_send(message.clone()) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Failed to send message to client: {}", e);
            }
        }
        Some(())
    }
}

impl Actor for WsChatServer {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.subscribe_system_async::<LeaveServer>(ctx);
        self.subscribe_system_async::<SendMessage>(ctx);
    }
}

impl Handler<JoinServer> for WsChatServer {
    type Result = MessageResult<JoinServer>;

    fn handle(&mut self, msg: JoinServer, _ctx: &mut Self::Context) -> Self::Result {
        // let JoinServer(server_name, client_name, client) = msg;

        let id = self.add_client_to_server(&msg.server_name, None, msg.client.clone());
        let join_msg = format!(
            "{} joined {}",
            msg.client_name.unwrap_or_else(|| "anon".to_owned()),
            msg.server_name,
        );
        self.send_system_message(&msg.server_name, &join_msg, msg.client);
        MessageResult(id)
    }
}

impl Handler<LeaveServer> for WsChatServer {
    type Result = ();

    fn handle(&mut self, msg: LeaveServer, _ctx: &mut Self::Context) {
        if let Some(server) = self.server_client_connections.get_mut(&msg.0) {
            server.remove(&msg.1);
        }
    }
}

impl Handler<ListServer> for WsChatServer {
    type Result = MessageResult<ListServer>;

    fn handle(&mut self, _: ListServer, _ctx: &mut Self::Context) -> Self::Result {
        MessageResult(self.server_client_connections.keys().cloned().collect())
    }
}

impl Handler<SendMessage> for WsChatServer {
    type Result = ();

    fn handle(&mut self, msg: SendMessage, _ctx: &mut Self::Context) {
        let SendMessage(server_name, id, msg) = msg;
        self.send_chat_message(&server_name, &msg, id);
    }
}

impl SystemService for WsChatServer {}
impl Supervised for WsChatServer {}
