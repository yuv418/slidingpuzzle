use ggez::{
    glam::Vec2,
    graphics::{self, Color, DrawMode, Mesh, PxScale, Rect, Text, TextFragment},
    Context, GameResult,
};
use keyframe::{functions::EaseInOut, keyframes, AnimationSequence};

use crate::game::{drawable::Drawable, scene::Scene};

// We use a struct since it might help later for stuff such as colours
// This will become drawable and let us control puzzle listings/settings
// and hold the meshes for drawing whatever
pub struct GameMenuMapping {
    pub text: String,
    pub next_page: Box<dyn Fn(&mut Context) -> Box<dyn Scene>>,

    // Positioning
    x: f32,
    y: f32,
    w: f32,
    h: f32,

    // Animation
    select_animation: Option<AnimationSequence<f32>>,

    // Meshes
    text_box_rect: Mesh,
    menu_text: Text,

    // For use with select() and deselect() and drawing animations
    currently_selected: bool,
}

impl GameMenuMapping {
    pub fn new(
        ctx: &mut Context,
        text: &str,
        next_page: Box<dyn Fn(&mut Context) -> Box<dyn Scene>>,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    ) -> GameResult<Self> {
        Ok(Self {
            text: text.to_string(),
            next_page,
            x,
            y,
            w,
            h,
            select_animation: None,
            menu_text: Text::new(TextFragment {
                text: text.to_string(),
                color: Some(Color::WHITE),
                font: Some("SecularOne-Regular".into()),
                scale: Some(PxScale::from(48.0)),
            }),
            text_box_rect: Mesh::new_rounded_rectangle(
                ctx,
                DrawMode::fill(),
                Rect {
                    x: 0.0,
                    y: 0.0,
                    w,
                    h,
                },
                8.0,
                Color::BLACK,
            )?,
            currently_selected: false,
        })
    }

    pub fn select(&mut self) {
        self.currently_selected = true;
        self.select_animation = Some(keyframes![
            (0.0, 0.0, EaseInOut),
            (self.w - 10.0, 0.5, EaseInOut)
        ]);
        for frag in self.menu_text.fragments_mut() {
            frag.color = Some(Color::BLACK);
        }
    }
    pub fn deselect(&mut self) {
        self.currently_selected = false;
        self.select_animation = Some(keyframes![
            (self.w - 10.0, 0.0, EaseInOut),
            (0.0, 0.5, EaseInOut)
        ]);
        for frag in self.menu_text.fragments_mut() {
            frag.color = Some(Color::WHITE);
        }
    }
}

impl Drawable for GameMenuMapping {
    fn draw(&mut self, ctx: &mut Context, canvas: &mut ggez::graphics::Canvas) -> ggez::GameResult {
        canvas.draw(&self.text_box_rect, Vec2::new(self.x, self.y));
        if self.currently_selected || self.select_animation.is_some() {
            let mut delete_animation = false;
            let selection_box = Mesh::new_rounded_rectangle(
                ctx,
                DrawMode::fill(),
                Rect {
                    x: 5.0,
                    y: self.y + 5.0,
                    w: if let Some(se) = &mut self.select_animation {
                        se.advance_by(0.05);
                        if se.finished() {
                            delete_animation = true;
                        }
                        se.now()
                    } else {
                        self.w - 10.0
                    },
                    h: self.h - 10.0,
                },
                5.0,
                Color::WHITE,
            )?;
            if delete_animation {
                self.select_animation = None;
            }
            canvas.draw(&selection_box, Vec2::new(self.x, 0.0));
        }

        let mt_sz = self
            .menu_text
            .measure(ctx)
            .expect("Failed to calculate menu text size");
        let mt_y = self.y + ((80.0 - mt_sz.y) / 2.0);

        canvas.draw(
            &self.menu_text,
            graphics::DrawParam::from([self.x + 20.0, mt_y]),
        );

        Ok(())
    }
}
