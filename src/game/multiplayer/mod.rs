use serde::{Deserialize, Serialize};

pub mod join_scene;
pub mod transport;

#[derive(Serialize, Deserialize, Debug)]
pub enum MultiplayerGameMessage {
    ConnectionString(String),
}
