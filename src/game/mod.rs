use ggez::event;
use ggez::graphics::{self, Image};
use ggez::{Context, GameResult};
use glam::*;
use rand::Rng;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use self::drawable::Drawable;

pub mod drawable;
pub mod tiles;

const TILE_SLIDE_DURATION: f32 = 0.2;

pub struct GameState {
    pub tile_state: tiles::TileState,
    pub set_winsize: bool,
}

impl GameState {
    pub fn new(img_path: PathBuf, tile_size: u32, context: &mut Context) -> GameResult<Self> {
        // Loop through and make the tiles
        Ok(Self {
            tile_state: tiles::TileState::new(context, img_path, tile_size, 0.0, 0.0)?,
            set_winsize: false,
        })
    }
}

impl event::EventHandler<ggez::GameError> for GameState {
    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        keycode: event::KeyCode,
        _keymods: event::KeyMods,
        repeat: bool,
    ) {
        let i = self.tile_state.blank_cell.0;
        let j = self.tile_state.blank_cell.1;
        if !repeat && !self.tile_state.game_completed() {
            // TODO make this DRYer
            match keycode {
                event::KeyCode::Up => {
                    // Tile below space
                    if i + 1 < self.tile_state.ref_board.len() {
                        self.tile_state
                            .swap_ref_tiles((i, j), (i + 1, j), TILE_SLIDE_DURATION);
                    }
                }
                event::KeyCode::Down => {
                    // Tile above space
                    if i != 0 {
                        self.tile_state
                            .swap_ref_tiles((i, j), (i - 1, j), TILE_SLIDE_DURATION);
                    }
                }
                event::KeyCode::Left => {
                    // Tile left of space
                    if j + 1 < self.tile_state.ref_board[i].len() {
                        self.tile_state
                            .swap_ref_tiles((i, j), (i, j + 1), TILE_SLIDE_DURATION);
                    }
                }
                event::KeyCode::Right => {
                    // Tile right of space
                    if j != 0 {
                        self.tile_state
                            .swap_ref_tiles((i, j), (i, j - 1), TILE_SLIDE_DURATION);
                    }
                }
                _ => {}
            }
            self.tile_state.check_completed();
        }
    }
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        // self.pos_x = self.pos_x % 800.0 + 20.0;
        Ok(())
    }
    fn draw(&mut self, ctx: &mut ggez::Context) -> GameResult {
        graphics::clear(ctx, [1.0, 1.0, 1.0, 1.0].into());
        self.tile_state.draw(ctx)
    }
}
