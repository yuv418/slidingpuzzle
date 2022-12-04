use ggez::{
    glam::Vec2,
    graphics::{self, Color, DrawMode, Drawable, Image, Mesh, PxScale, Rect, Text, TextFragment},
    Context, GameResult,
};
use keyframe::{functions::EaseInOut, keyframes, AnimationSequence};

use crate::game::{drawable::Drawable as SlidingPuzzleDrawable, scene::Scene};

pub enum GameMenuItemVariant {
    // i.e a button
    TextItem {
        text_mesh: Text,
    },
    NumberInput {
        number: u32,
        prompt_mesh: Text,
        number_mesh: Text,

        // [ prompt [ number ] ] -> inner brackets are what this is for
        number_highlight_rect: Mesh,
    },

    ImageItem {
        image: Image,
        caption_mesh: Text,
    },
}

// We use a struct since it might help later for stuff such as colours
// This will become drawable and let us control puzzle listings/settings
// and hold the meshes for drawing whatever
pub struct GameMenuItem {
    pub next_page: Box<dyn Fn(&mut Context) -> Box<dyn Scene>>,

    // Positioning
    x: f32,
    y: f32,
    w: f32,
    h: f32,

    // Animation
    select_animation: Option<AnimationSequence<f32>>,

    // Meshes
    item_box_rect: Mesh,

    item_variant: GameMenuItemVariant,

    // For use with select() and deselect() and drawing animations
    currently_selected: bool,
}

impl GameMenuItem {
    pub fn new_text_item(
        ctx: &mut Context,
        text: &str,
        next_page: Box<dyn Fn(&mut Context) -> Box<dyn Scene>>,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    ) -> GameResult<Self> {
        Self::new(
            ctx,
            next_page,
            GameMenuItemVariant::TextItem {
                text_mesh: Text::new(TextFragment {
                    text: text.to_string(),
                    color: Some(Color::WHITE),
                    font: Some("SecularOne-Regular".into()),
                    scale: Some(PxScale::from(48.0)),
                }),
            },
            x,
            y,
            w,
            h,
        )
    }
    pub fn new_image_item(
        ctx: &mut Context,
        image: Image,
        caption: &str,
        next_page: Box<dyn Fn(&mut Context) -> Box<dyn Scene>>,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    ) -> GameResult<Self> {
        Self::new(
            ctx,
            next_page,
            GameMenuItemVariant::ImageItem {
                image,
                caption_mesh: Text::new(TextFragment {
                    text: caption.to_string(),
                    color: Some(Color::WHITE),
                    font: Some("SecularOne-Regular".into()),
                    scale: Some(PxScale::from(28.0)),
                }),
            },
            x,
            y,
            w,
            h,
        )
    }

    fn new(
        ctx: &mut Context,
        next_page: Box<dyn Fn(&mut Context) -> Box<dyn Scene>>,
        item_variant: GameMenuItemVariant,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    ) -> GameResult<Self> {
        Ok(Self {
            next_page,
            item_variant,
            x,
            y,
            w,
            h,
            item_box_rect: Mesh::new_rounded_rectangle(
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
            select_animation: None,
            currently_selected: false,
        })
    }

    pub fn select(&mut self) {
        self.currently_selected = true;
        self.select_animation = Some(keyframes![
            (0.0, 0.0, EaseInOut),
            (self.w - 10.0, 0.5, EaseInOut)
        ]);
        match &mut self.item_variant {
            GameMenuItemVariant::TextItem { text_mesh } => {
                for frag in text_mesh.fragments_mut() {
                    frag.color = Some(Color::BLACK);
                }
            }
            GameMenuItemVariant::NumberInput { .. } => todo!(),
            GameMenuItemVariant::ImageItem { caption_mesh, .. } => {
                for frag in caption_mesh.fragments_mut() {
                    frag.color = Some(Color::BLACK);
                }
            }
        }
    }
    pub fn deselect(&mut self) {
        self.currently_selected = false;
        self.select_animation = Some(keyframes![
            (self.w - 10.0, 0.0, EaseInOut),
            (0.0, 0.5, EaseInOut)
        ]);
        match &mut self.item_variant {
            GameMenuItemVariant::TextItem { text_mesh } => {
                for frag in text_mesh.fragments_mut() {
                    frag.color = Some(Color::WHITE);
                }
            }
            GameMenuItemVariant::NumberInput { .. } => todo!(),
            GameMenuItemVariant::ImageItem { caption_mesh, .. } => {
                for frag in caption_mesh.fragments_mut() {
                    frag.color = Some(Color::WHITE);
                }
            }
        }
    }
}

impl SlidingPuzzleDrawable for GameMenuItem {
    fn draw(&mut self, ctx: &mut Context, canvas: &mut ggez::graphics::Canvas) -> ggez::GameResult {
        canvas.draw(&self.item_box_rect, Vec2::new(self.x, self.y));
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

        match &self.item_variant {
            GameMenuItemVariant::TextItem { text_mesh } => {
                let mt_sz = text_mesh
                    .measure(ctx)
                    .expect("Failed to calculate menu text size");
                let mt_y = self.y + ((80.0 - mt_sz.y) / 2.0);

                canvas.draw(text_mesh, graphics::DrawParam::from([self.x + 20.0, mt_y]));
            }
            GameMenuItemVariant::NumberInput {
                number,
                prompt_mesh,
                number_mesh,
                number_highlight_rect,
            } => todo!(),
            GameMenuItemVariant::ImageItem {
                image,
                caption_mesh,
            } => {
                // Want image to be self.w - 60.0

                let scale_factor = (self.w - 60.0) / image.width() as f32;

                canvas.draw(
                    image,
                    // We have to scale the image to fit in the box
                    graphics::DrawParam::from([self.x + 20.0, self.y + 20.0])
                        .scale(Vec2::from((scale_factor, scale_factor))),
                );
                canvas.draw(
                    caption_mesh,
                    graphics::DrawParam::from([
                        self.x + 20.0,
                        self.y + (image.height() as f32 * scale_factor) + 25.0,
                    ]),
                );
            }
        }

        Ok(())
    }
}
