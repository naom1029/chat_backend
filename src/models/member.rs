use serde::{Deserialize, Serialize};
use uuid::Uuid; // Userモデルをインポート
use chrono::{DateTime, Utc};

// サーバー内のメンバーモデル
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Member {
    pub id: Uuid,      // メンバーID（サーバー内での一意の識別子、必須ではないが追加可能）
    pub user_id: Uuid, // グローバルなユーザーID（User.id への参照）
    pub nickname: Option<String>, // サーバー固有のニックネーム
    // pub roles: Vec<Uuid>, // 割り当てられた役割（Role IDのリスト）
    pub joined_at: DateTime<Utc>, // サーバーに参加した日時
}

impl Member {
    pub fn new(user_id: Uuid) -> Self {
        Member {
            id: Uuid::new_v4(),
            user_id,
            nickname: None,
            joined_at: Utc::now(),
        }
    }
}
