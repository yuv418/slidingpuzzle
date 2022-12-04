use self::scene::Scene;
use ggez::event;
use ggez::graphics;
use ggez::input::keyboard::KeyInput;
use ggez::{Context, GameResult};
use glam::*;

pub mod drawable;
pub mod gmenu;
pub mod player;
pub mod scene;
pub mod tiles;

pub struct GameState {
    pub current_scene: Box<dyn Scene>,
    pub set_winsize: bool,
}

impl GameState {
    pub fn new(context: &mut Context) -> GameResult<Self> {
        // Loop through and make the tiles
        Ok(Self {
            current_scene: Box::new(gmenu::GameMenu::new(context)),
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
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        if let Some(next_scene) = self.current_scene.next_scene(ctx) {
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
