use ggez::{Context, GameResult};

pub trait Drawable {
    fn draw(&mut self, ctx: &mut Context) -> GameResult;
}
