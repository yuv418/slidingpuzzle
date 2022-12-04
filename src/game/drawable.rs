use ggez::{graphics::Canvas, Context, GameResult};

pub trait Drawable {
    fn draw(&mut self, ctx: &mut Context, canvas: &mut Canvas) -> GameResult;
}
