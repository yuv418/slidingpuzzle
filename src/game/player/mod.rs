use std::collections::BinaryHeap;

use ggez::{Context, GameResult};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct PlayerSettings {
    pub num_rows_cols: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Player {
    id: Uuid,
    username: String,
    pub completed_puzzles: BinaryHeap<usize>,
    pub player_settings: PlayerSettings,
}

impl Player {
    pub fn load(ctx: &mut Context) -> GameResult<Self> {
        let save_file = ctx.fs.open("/player.dat")?;
        bincode::deserialize_from(save_file)
            .map_err(|_| ggez::GameError::FilesystemError("Failed to read player.dat".to_string()))
    }
    pub fn save(&self, ctx: &mut Context) -> GameResult {
        let save_file = ctx.fs.create("/player.dat")?;
        bincode::serialize_into(save_file, self)
            .map_err(|_| ggez::GameError::FilesystemError("Failed to save player.dat".to_string()))
    }
    pub fn new(username: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            username,
            completed_puzzles: BinaryHeap::new(),
            player_settings: PlayerSettings { num_rows_cols: 3 },
        }
    }
}
