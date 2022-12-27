use std::sync::Arc;

use ggez::{GameError, GameResult};
use log::trace;

use crate::game::{
    multiplayer::{transport::MultiplayerTransport, MultiplayerGameMessage},
    player::PuzzleStatistics,
};

#[derive(Default)]
pub struct TileMultiplayerTransport {
    transport: Option<Arc<MultiplayerTransport>>,
}

impl TileMultiplayerTransport {
    pub fn new(transport: Option<Arc<MultiplayerTransport>>) -> Self {
        Self { transport }
    }

    pub fn delete_random_tile(&mut self, tile: (usize, usize)) -> GameResult {
        if let Some(t) = &self.transport {
            t.event_push_buffer
                .send(MultiplayerGameMessage::DeleteRandomTile(tile))
                .map_err(|_| {
                    GameError::CustomError("Failed to send delete random tile to peer".to_string())
                })?;
        }
        Ok(())
    }

    pub fn swap_tiles(&mut self, i1j1: (usize, usize), i2j2: (usize, usize), duration: f32) {
        if let Some(transport) = &self.transport {
            // TODO handle this expect better
            transport
                .event_push_buffer
                .send(MultiplayerGameMessage::SwapTiles { i1j1, i2j2, duration })
                .expect("Failed to push tile swap into transport");
        }
    }

    pub fn recv_message(&mut self) -> Option<MultiplayerGameMessage> {
        if let Some(transport) = &self.transport {
            match transport.event_buffer.try_recv() {
                Ok(msg) => {
                    trace!("recv tile msg {:?}", msg);
                    Some(msg)
                }
                _ => None,
            }
        } else {
            None
        }
    }

    pub fn end_game(&mut self, stats: PuzzleStatistics) {
        if let Some(transport) = &self.transport {
            // TODO handle this expect better
            transport
                .event_push_buffer
                .send(MultiplayerGameMessage::GameCompleted(stats))
                .expect("Failed to push game completed in transport");
        }
    }
}
