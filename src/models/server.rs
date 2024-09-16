use uuid::Uuid;
use std::collections::HashMap;
use crate::models::member::Member;

// サーバーモデル
pub struct Server {
    pub id: Uuid,
    pub name: String,
    pub channels: HashMap<Uuid, Channel>,
    members: HashMap<Uuid, Member>,
}

impl Server {
    pub fn new(name: String) -> Self {
        Server {
            id: Uuid::new_v4(),
            name,
            channels: HashMap::new(),
            members: HashMap::new(),
        }
    }

    pub fn add_member(&mut self, member: Member) {
        self.members.insert(member.id, member);
    }
}

// チャンネルモデルの仮定
pub struct Channel {
    pub id: Uuid,
    pub name: String,
}

impl Channel {
    pub fn new(name: String) -> Self {
        Channel {
            id: Uuid::new_v4(),
            name,
        }
    }
}
