use serde::{Deserialize, Serialize};

pub mod transport;

#[derive(Serialize, Deserialize, Debug)]
pub struct MultiplayerGameMessage {}
