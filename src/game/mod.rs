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
    pub current_scene: Box<dyn Scene>,
    pub set_winsize: bool,
}

impl GameState {
    pub fn new(img_path: PathBuf, tile_size: u32, context: &mut Context) -> GameResult<Self> {
        // Loop through and make the tiles
        Ok(Self {
            current_scene: Box::new(gmenu::GameMenu::new()),
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
        self.current_scene.handle_key_event(ctx, key_input, repeat);
        Ok(())
    }
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        if let Some(next_scene) = self.current_scene.next_scene() {
            self.current_scene = next_scene;
        }
        Ok(())
    }
    fn draw(&mut self, ctx: &mut ggez::Context) -> GameResult {
        let mut canvas =
            graphics::Canvas::from_frame(ctx, graphics::Color::from([1.0, 1.0, 1.0, 1.0]));
        self.current_scene.draw(ctx, &mut canvas)?;
        canvas.finish(ctx)
    }
}
