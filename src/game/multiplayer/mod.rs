use serde::{Deserialize, Serialize};

use super::player::PuzzleStatistics;

pub mod game_view;
pub mod join_scene;
pub mod transport;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum MultiplayerGameMessage {
    ConnectionString(String),
    Hello { username: String },
    CloseConnection,
    DeleteRandomTile((usize, usize)),
    StartGame { img_num: usize, num_rows_cols: usize, host_username: String },
    SwapTiles { i1j1: (usize, usize), i2j2: (usize, usize), duration: f32 },
    ScramblingFinished,
    GameCompleted(PuzzleStatistics),
}
