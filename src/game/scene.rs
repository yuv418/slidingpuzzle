use ggez::{graphics::Canvas, input::keyboard::KeyInput, Context, GameResult};

use super::drawable::Drawable;

//
pub trait Scene: Drawable {
    fn handle_key_event(&mut self, _ctx: &mut Context, key_input: KeyInput, repeat: bool);
    fn next_scene(&self, ctx: &mut Context) -> Option<Box<dyn Scene>>;
    // To use when the scene is transitioning to/from the next scene
    fn draw_transition(&mut self, ctx: &mut Context, canvas: &mut Canvas) -> GameResult;
}
