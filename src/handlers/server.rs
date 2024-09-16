use crate::models::user::User;
use crate::models::{ChatMessage, JoinServer, LeaveServer, ListServer, SendMessage};
use actix::prelude::*;
use actix_broker::BrokerSubscribe;
use actix_web::{
    web::{self, Data},
    Error, HttpRequest, HttpResponse,
};
use actix_web_actors::ws::{self, WebsocketContext};
use std::sync::atomic::AtomicUsize;
use std::{collections::HashMap, sync::Arc};

use log::{error, info, warn};
use tokio::sync::RwLock;
use uuid::Uuid;
#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct Message(pub String);

// ユーザーリストの型定義
type Users = Arc<RwLock<HashMap<Uuid, HashMap<usize, (User, Addr<WsChatServer>)>>>>;
// 接続IDを生成するためのカウンター
static NEXT_CONN_ID: AtomicUsize = AtomicUsize::new(1);

type Client = Recipient<ChatMessage>;
type Server = HashMap<Uuid, Client>;

#[derive(Default)]
pub struct WsChatServer {
    servers: HashMap<String, Server>,
}

impl WsChatServer {
    fn take_server(&mut self, server_name: &str) -> Option<Server> {
        let server = self.servers.get_mut(server_name)?;
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

        if let Some(server) = self.servers.get_mut(server_name) {
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
        let mut server: Server = HashMap::new();

        server.insert(id, client);
        self.servers.insert(server_name.to_owned(), server);

        id
    }
    fn send_chat_message(&mut self, server_name: &str, msg: &str, _src: Uuid) -> Option<()> {
        let mut server = self.take_server(server_name)?;
        let message = ChatMessage(msg.to_owned()); // 一度だけクローン

        for (id, client) in server.drain() {
            match client.try_send(message.clone()) {
                Ok(_) => {
                    self.add_client_to_server(server_name, Some(id), client);
                }
                Err(e) => {
                    eprintln!("Failed to send message to client {}: {}", id, e);
                }
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
        let JoinServer(server_name, client_name, client) = msg;

        let id = self.add_client_to_server(&server_name, None, client);
        let join_msg = format!(
            "{} joined {server_name}",
            client_name.unwrap_or_else(|| "anon".to_owned()),
        );

        self.send_chat_message(&server_name, &join_msg, id);
        MessageResult(id)
    }
}

impl Handler<LeaveServer> for WsChatServer {
    type Result = ();

    fn handle(&mut self, msg: LeaveServer, _ctx: &mut Self::Context) {
        if let Some(server) = self.servers.get_mut(&msg.0) {
            server.remove(&msg.1);
        }
    }
}

impl Handler<ListServer> for WsChatServer {
    type Result = MessageResult<ListServer>;

    fn handle(&mut self, _: ListServer, _ctx: &mut Self::Context) -> Self::Result {
        MessageResult(self.servers.keys().cloned().collect())
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
