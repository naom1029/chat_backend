use crate::handlers::server::WsChatServer;
use crate::models::message::{
    ChatMessage, ClientMessage, CommandMessage, JoinServer, ListServer, SendMessage, ServerMessage,
};
use actix::prelude::*;
use actix_broker::BrokerIssue;
use actix_web_actors::ws;
use log::error;
use uuid::Uuid;

#[derive(Default)]
pub struct WsChatSession {
    id: Uuid,
    room: String,
    name: Option<String>,
}

impl WsChatSession {
    pub fn join_server(&mut self, server_name: &str, ctx: &mut ws::WebsocketContext<Self>) {
        let server_name = server_name.to_owned();

        // Then send a join message for the new room
        let join_msg = JoinServer {
            server_name: server_name.to_owned(),
            client_name: self.name.clone(),
            client: ctx.address().recipient(),
        };
        WsChatServer::from_registry()
            .send(join_msg)
            .into_actor(self)
            .then(|id, act, _ctx| {
                if let Ok(id) = id {
                    act.id = id;
                    act.room = server_name;
                }

                fut::ready(())
            })
            .wait(ctx);
    }
    pub fn list_server(&mut self, ctx: &mut ws::WebsocketContext<Self>) {
        WsChatServer::from_registry()
            .send(ListServer)
            .into_actor(self)
            .then(|res, _, ctx| {
                if let Ok(rooms) = res {
                    for room in rooms {
                        ctx.text(room);
                    }
                }

                fut::ready(())
            })
            .wait(ctx);
    }

    pub fn send_msg(&mut self, msg: ClientMessage, ctx: &mut ws::WebsocketContext<Self>) {
        match msg {
            ClientMessage::Chat(chat_msg) => {
                let server_message = SendMessage(self.room.clone(), self.id, chat_msg.text);
                self.issue_system_async(server_message);
            }
            ClientMessage::Command(cmd_msg) => {
                self.handle_command(cmd_msg, ctx);
            }
        }
    }

    fn handle_command(&mut self, cmd_msg: CommandMessage, ctx: &mut ws::WebsocketContext<Self>) {
        match Some(cmd_msg.command.as_str()) {
            Some("list") => self.list_server(ctx),

            Some("join") => {
                if let Some(server_name) = cmd_msg.args.as_deref() {
                    self.join_server(server_name, ctx);
                } else {
                    ctx.text("!!! room name is required");
                }
            }

            Some("name") => {
                if let Some(name) = cmd_msg.args {
                    self.name = Some(name.to_owned());
                    ctx.text(format!("name changed to: {name}"));
                } else {
                    ctx.text("!!! name is required");
                }
            }

            _ => ctx.text(format!("!!! unknown command: {cmd_msg:?}")),
        }
    }
}

impl Actor for WsChatSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // self.join_server("Server1", ctx);
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        log::info!(
            "WsChatSession closed for {}({}) in room {}",
            self.name.clone().unwrap_or_else(|| "anon".to_owned()),
            self.id,
            self.room
        );
    }
}
impl Handler<ServerMessage> for WsChatSession {
    type Result = ();

    fn handle(&mut self, msg: ServerMessage, ctx: &mut Self::Context) {
        let json_str = match msg {
            ServerMessage::Chat(chat_msg) => serde_json::to_string(&chat_msg).unwrap_or_else(|e| {
                eprintln!("ChatMessage のシリアル化に失敗しました: {}", e);
                "{}".to_owned()
            }),
            ServerMessage::System(sys_msg) => serde_json::to_string(&sys_msg).unwrap_or_else(|e| {
                eprintln!("SystemMessage のシリアル化に失敗しました: {}", e);
                "{}".to_owned()
            }),
        };
        ctx.text(json_str);
    }
}

// クライアント側からの受信メッセージを処理
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsChatSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let msg = match msg {
            Err(e) => {
                error!("WebSocket エラー: {:?}", e);
                ctx.stop();
                return;
            }
            Ok(msg) => msg,
        };

        log::debug!("WEBSOCKET MESSAGE: {:?}", msg);

        match msg {
            ws::Message::Text(text) => {
                let trimmed = text.trim();

                // JSON形式のメッセージをデシリアライズ
                match serde_json::from_str::<ClientMessage>(trimmed) {
                    Ok(client_msg) => {
                        self.send_msg(client_msg, ctx);
                    }
                    Err(e) => {
                        error!("ClientMessage のデシリアライズに失敗しました: {}", e);
                        ctx.text("!!! invalid message format");
                    }
                }
            }
            ws::Message::Binary(_) => {
                ctx.text("!!! binary messages are not supported");
            }
            ws::Message::Ping(msg) => {
                ctx.pong(&msg);
            }
            ws::Message::Pong(_) => {}
            ws::Message::Close(reason) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => {}
        }
    }
}
