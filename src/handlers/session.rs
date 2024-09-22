use crate::handlers::server::WsChatServer;
use crate::models::message::{ClientMessage, JoinServer, ListServer, SendMessage, ServerMessage};
use actix::prelude::*;
use actix_broker::BrokerIssue;
use actix_web_actors::ws;
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
        let join_msg = JoinServer(
            server_name.to_owned(),
            self.name.clone(),
            ctx.address().recipient(),
        );
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

    pub fn send_msg(&self, msg: ClientMessage) {
        let content = format!(
            "{}: {}",
            self.name.clone().unwrap_or_else(|| "anon".to_owned()),
            msg.text
        );

        let msg = SendMessage(self.room.clone(), self.id, content);

        self.issue_system_async(msg);
    }
}

impl Actor for WsChatSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.join_server("main", ctx);
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
        ctx.text(msg.text);
    }
}
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsChatSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let msg = match msg {
            Err(_) => {
                ctx.stop();
                return;
            }
            Ok(msg) => msg,
        };

        log::debug!("WEBSOCKET MESSAGE: {msg:?}");

        match msg {
            ws::Message::Text(text) => {
                let msg = text.trim();

                // コマンドメッセージ
                if msg.starts_with('/') {
                    let mut command = msg.splitn(2, ' ');

                    match command.next() {
                        Some("/list") => self.list_server(ctx),

                        Some("/join") => {
                            if let Some(server_name) = command.next() {
                                self.join_server(server_name, ctx);
                            } else {
                                ctx.text("!!! room name is required");
                            }
                        }

                        Some("/name") => {
                            if let Some(name) = command.next() {
                                self.name = Some(name.to_owned());
                                ctx.text(format!("name changed to: {name}"));
                            } else {
                                ctx.text("!!! name is required");
                            }
                        }

                        _ => ctx.text(format!("!!! unknown command: {msg:?}")),
                    }

                    return;
                }
                if let Ok(chat_message) = serde_json::from_str::<ClientMessage>(msg) {
                    self.send_msg(chat_message);
                } else {
                    ctx.text("!!! invalid message format");
                }
            }
            ws::Message::Close(reason) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => {}
        }
    }
}
