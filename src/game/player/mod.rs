use std::{collections::BTreeMap, sync::Mutex, time::Duration};

use chrono::{DateTime, Local};
use ggez::{Context, GameResult};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod settings_scene;

use lazy_static::lazy_static;

// TODO use a parking lot Mutex
lazy_static! {
    pub static ref PLAYER: Mutex<Option<Player>> = Mutex::new(None);
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PlayerSettings {
    pub num_rows_cols: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PuzzleStatistics {
    pub finish_time: DateTime<Local>,
    pub duration: Duration,
    pub move_count: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Player {
    id: Uuid,
    username: String,
    pub completed_puzzles: BTreeMap<usize, Vec<PuzzleStatistics>>,
    pub player_settings: PlayerSettings,
}

impl Player {
    pub fn username(&self) -> String {
        self.username.clone()
    }
    pub fn load(ctx: &mut Context) -> GameResult<Self> {
        let save_file = ctx.fs.open("/player.dat")?;
        bincode::deserialize_from(save_file).map_err(|_| ggez::GameError::FilesystemError("Failed to read player.dat".to_string()))
    }
    pub fn save(&self, ctx: &mut Context) -> GameResult {
        let save_file = ctx.fs.create("/player.dat")?;
        bincode::serialize_into(save_file, self).map_err(|_| ggez::GameError::FilesystemError("Failed to save player.dat".to_string()))
    }
    pub fn new(username: String, player_settings: PlayerSettings) -> Self {
        Self { id: Uuid::new_v4(), username, completed_puzzles: BTreeMap::new(), player_settings }
    }
    pub fn startup(ctx: &mut Context) -> bool {
        let mut opt_player = PLAYER.lock().unwrap();
        let loaded_player = Player::load(ctx);

        match loaded_player {
            Err(_) => true,
            Ok(p) => {
                *opt_player = Some(p);
                false
            }
        }
    }
}
