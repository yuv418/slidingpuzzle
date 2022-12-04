use ggez::{Context, GameResult};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct Player {
    id: Uuid,
    username: String,
    completed_puzzles: usize,
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
            completed_puzzles: 0,
        }
    }
}
