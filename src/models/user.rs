use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ユーザーモデル
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
}

impl User {
    pub fn new() -> Self {
        User {
            id: Uuid::new_v4(),
            username: String::new(),
            email: String::new(),
            password_hash: String::new(),
            created_at: Utc::now(),
        }
    }

    pub async fn get_user(&self, _id: Uuid) -> Option<Self> {
        // データベースやストレージからユーザーを取得するロジックを実装
        Some(self.clone())
    }
}
