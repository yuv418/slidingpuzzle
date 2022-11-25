use ggez::{Context, GameResult};

pub trait Drawable {
    fn draw(&mut self, ctx: &mut Context, x: f32, y: f32) -> GameResult;
}
