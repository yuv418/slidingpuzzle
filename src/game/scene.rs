use ggez::{graphics::Canvas, Context, GameResult};

use super::{drawable::Drawable, input::InputAction};

//
pub trait Scene: Drawable {
    fn handle_input_event(&mut self, _ctx: &mut Context, _key_input: InputAction) {}
    fn next_scene(&mut self, _ctx: &mut Context) -> Option<Box<dyn Scene>> { None }
    // To use when the scene is transitioning to/from the next scene
    fn draw_transition(&mut self, ctx: &mut Context, canvas: &mut Canvas) -> GameResult { self.draw(ctx, canvas) }
    fn text_input_event(&mut self, _ctx: &mut ggez::Context, _c: char) {}
    fn update(&mut self, _ctx: &mut Context) -> GameResult { Ok(()) }
}
