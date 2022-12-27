use ggez::{
    glam::Vec2,
    graphics::{Color, PxScale, Text, TextFragment},
};

use crate::game::{
    animation::{animatable::Animatable, DrawablePos},
    drawable::Drawable,
};

pub struct UIText {
    pub text: Text,
    pub pos: DrawablePos,
}

impl UIText {
    pub fn new(text: String, color: Color, size: f32, pos: DrawablePos) -> Self {
        Self {
            text: Text::new(TextFragment {
                text,
                font: Some("SecularOne-Regular".into()),
                scale: Some(PxScale::from(size)),
                color: Some(color),
            }),
            pos,
        }
    }
}

impl Drawable for UIText {
    fn draw(
        &mut self,
        ctx: &mut ggez::Context,
        canvas: &mut ggez::graphics::Canvas,
    ) -> ggez::GameResult {
        canvas.draw(&self.text, Vec2::new(self.pos.x, self.pos.y));
        Ok(())
    }
}

impl Animatable<DrawablePos> for UIText {
    fn set_state(&mut self, now: DrawablePos) {
        self.pos = now;
    }
}
