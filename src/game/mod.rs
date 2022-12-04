use ggez::event;
use ggez::graphics::{self, Image};
use ggez::input::keyboard::KeyInput;
use ggez::{Context, GameResult};
use glam::*;
use rand::Rng;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use self::drawable::Drawable;
use self::scene::Scene;

pub mod drawable;
pub mod gmenu;
pub mod scene;
pub mod tiles;

pub struct GameState {
    pub tile_state: Option<tiles::TileState>,
    pub game_menu: Option<gmenu::GameMenu>,
    pub set_winsize: bool,
}

impl GameState {
    pub fn new(img_path: PathBuf, tile_size: u32, context: &mut Context) -> GameResult<Self> {
        // Loop through and make the tiles
        Ok(Self {
            game_menu: Some(gmenu::GameMenu::new()),
            // tile_state: Some(tiles::TileState::new(context, img_path, tile_size, 0.0, 0.0)?),
            tile_state: None,
            set_winsize: false,
        })
    }
}

impl event::EventHandler<ggez::GameError> for GameState {
    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        key_input: KeyInput,
        repeat: bool,
    ) -> GameResult {
        if let Some(gmenu) = &mut self.game_menu {
            gmenu.handle_key_event(ctx, key_input, repeat);
        } else if let Some(tile_state) = &mut self.tile_state {
            tile_state.handle_key_event(ctx, key_input, repeat);
        }
        Ok(())
    }
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        // self.pos_x = self.pos_x % 800.0 + 20.0;
        Ok(())
    }
    fn draw(&mut self, ctx: &mut ggez::Context) -> GameResult {
        let mut canvas =
            graphics::Canvas::from_frame(ctx, graphics::Color::from([1.0, 1.0, 1.0, 1.0]));
        if let Some(gmenu) = &mut self.game_menu {
            gmenu.draw(ctx, &mut canvas);
        } else if let Some(tile_state) = &mut self.tile_state {
            tile_state.draw(ctx, &mut canvas);
        }
        canvas.finish(ctx)
    }
}
