use ggez::{
    glam::Vec2,
    graphics::{self, Color, DrawMode, Image, Mesh, Rect},
    Context, GameResult,
};
use keyframe::{functions::EaseInOut, keyframes, AnimationSequence};

use crate::game::{animation::DrawablePos, drawable::Drawable as SlidingPuzzleDrawable, scene::Scene, ui::uitext::UIText};

pub enum GameMenuItemVariant {
    // i.e a button
    TextItem {
        text_mesh: UIText,
    },
    InputItem {
        is_num: bool,
        text: String,

        prompt_mesh: UIText,

        // [ prompt [ text ] ] -> inner brackets are what this is for
        text_highlight_rect: Mesh,
        selected_text_highlight_rect: Mesh,
    },

    ImageItem {
        image: Image,
        scale_factor: f32,
        caption_mesh: UIText,
    },
}

// We use a struct since it might help later for stuff such as colours
// This will become drawable and let us control puzzle listings/settings
// and hold the meshes for drawing whatever
pub struct GameMenuItem {
    pub next_page: Option<Box<dyn Fn(&mut Context) -> Box<dyn Scene>>>,

    // Positioning
    // Public for keyframing purposes
    pub pos: DrawablePos,
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
    pub fn handle_input(&mut self, c: char) {
        if let GameMenuItemVariant::InputItem { is_num, text, .. } = &mut self.item_variant {
            // Backspace
            if c == '\x08' {
                text.pop();
            } else if c == '\n' || c == '\r' {
                // Prevent enters
                return;
            } else if !*is_num {
                text.push(c);
            } else if let Ok(_) = (text.to_owned() + &c.to_string()).parse::<u32>() {
                text.push(c);
            }
        }
    }

    // You'll have to parse the String to an int yourself ):
    pub fn get_input_value(&mut self) -> Option<String> {
        if let GameMenuItemVariant::InputItem { text, .. } = &self.item_variant {
            Some(text.to_string())
        } else {
            None
        }
    }

    pub fn new_input_item(
        ctx: &mut Context, prompt: &str, initial_value: String, is_num: bool,
        next_page: Option<Box<dyn Fn(&mut Context) -> Box<dyn Scene>>>, x: f32, y: f32, w: f32, h: f32,
    ) -> GameResult<Self> {
        let prompt_mesh = UIText::new(prompt.to_string(), Color::WHITE, 38.0, DrawablePos { x: x + 20.0, y: y + 10.0 });
        Self::new(
            ctx,
            next_page,
            GameMenuItemVariant::InputItem {
                prompt_mesh,
                is_num,
                text: initial_value,
                text_highlight_rect: Mesh::new_rounded_rectangle(
                    ctx,
                    DrawMode::fill(),
                    Rect { x: 0.0, y: 0.0, w: w * 0.8, h: 30.0 },
                    5.0,
                    Color::WHITE,
                )?,
                selected_text_highlight_rect: Mesh::new_rounded_rectangle(
                    ctx,
                    DrawMode::fill(),
                    Rect { x: 0.0, y: 0.0, w: w * 0.8, h: 30.0 },
                    5.0,
                    Color::BLACK,
                )?,
            },
            x,
            y,
            w,
            h,
        )
    }
    pub fn new_text_item(
        ctx: &mut Context, text: &str, next_page: Option<Box<dyn Fn(&mut Context) -> Box<dyn Scene>>>, x: f32, y: f32, w: f32, h: f32,
    ) -> GameResult<Self> {
        Self::new(
            ctx,
            next_page,
            GameMenuItemVariant::TextItem {
                text_mesh: UIText::new(
                    text.to_string(),
                    Color::WHITE,
                    48.0,
                    // This is never used. TODO kind of a hack.
                    Default::default(),
                ),
            },
            x,
            y,
            w,
            h,
        )
    }
    pub fn new_image_item(
        ctx: &mut Context, image: Image, caption: &str, next_page: Option<Box<dyn Fn(&mut Context) -> Box<dyn Scene>>>, x: f32, y: f32,
        w: f32, h: f32,
    ) -> GameResult<Self> {
        // Want image to be self.w - 60.0
        let scale_factor = (w - 60.0) / image.width() as f32;
        let mesh_y = y + (image.height() as f32 * scale_factor) + 25.0;

        Self::new(
            ctx,
            next_page,
            GameMenuItemVariant::ImageItem {
                image,
                scale_factor,
                caption_mesh: UIText::new(caption.to_string(), Color::WHITE, 28.0, DrawablePos { x: x + 20.0, y: mesh_y }),
            },
            x,
            y,
            w,
            h,
        )
    }

    fn new(
        ctx: &mut Context, next_page: Option<Box<dyn Fn(&mut Context) -> Box<dyn Scene>>>, item_variant: GameMenuItemVariant, x: f32,
        y: f32, w: f32, h: f32,
    ) -> GameResult<Self> {
        Ok(Self {
            next_page,
            item_variant,
            pos: DrawablePos { x, y },
            w,
            h,
            item_box_rect: Mesh::new_rounded_rectangle(ctx, DrawMode::fill(), Rect { x: 0.0, y: 0.0, w, h }, 8.0, Color::BLACK)?,
            select_animation: None,
            currently_selected: false,
        })
    }

    pub fn select(&mut self) {
        self.currently_selected = true;
        self.select_animation = Some(keyframes![(0.0, 0.0, EaseInOut), (self.w - 10.0, 0.5, EaseInOut)]);
        match &mut self.item_variant {
            GameMenuItemVariant::TextItem { text_mesh } => {
                for frag in text_mesh.text.fragments_mut() {
                    frag.color = Some(Color::BLACK);
                }
            }
            GameMenuItemVariant::InputItem { prompt_mesh, .. } => {
                for frag in prompt_mesh.text.fragments_mut() {
                    frag.color = Some(Color::BLACK);
                }
            }
            GameMenuItemVariant::ImageItem { caption_mesh, .. } => {
                for frag in caption_mesh.text.fragments_mut() {
                    frag.color = Some(Color::BLACK);
                }
            }
        }
    }
    pub fn deselect(&mut self) {
        self.currently_selected = false;
        self.select_animation = Some(keyframes![(self.w - 10.0, 0.0, EaseInOut), (0.0, 0.5, EaseInOut)]);
        match &mut self.item_variant {
            GameMenuItemVariant::TextItem { text_mesh } => {
                for frag in text_mesh.text.fragments_mut() {
                    frag.color = Some(Color::WHITE);
                }
            }

            GameMenuItemVariant::InputItem { prompt_mesh, .. } => {
                for frag in prompt_mesh.text.fragments_mut() {
                    frag.color = Some(Color::WHITE);
                }
            }
            GameMenuItemVariant::ImageItem { caption_mesh, .. } => {
                for frag in caption_mesh.text.fragments_mut() {
                    frag.color = Some(Color::WHITE);
                }
            }
        }
    }
}

impl SlidingPuzzleDrawable for GameMenuItem {
    fn draw(&mut self, ctx: &mut Context, canvas: &mut ggez::graphics::Canvas) -> ggez::GameResult {
        canvas.draw(&self.item_box_rect, Vec2::new(self.pos.x, self.pos.y));
        if self.currently_selected || self.select_animation.is_some() {
            let mut delete_animation = false;
            let selection_box = Mesh::new_rounded_rectangle(
                ctx,
                DrawMode::fill(),
                Rect {
                    x: 5.0,
                    y: self.pos.y + 5.0,
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
            canvas.draw(&selection_box, Vec2::new(self.pos.x, 0.0));
        }

        match &mut self.item_variant {
            GameMenuItemVariant::TextItem { text_mesh } => {
                let mt_sz = text_mesh.text.measure(ctx).expect("Failed to calculate menu text size");
                let mt_y = self.pos.y + ((80.0 - mt_sz.y) / 2.0);

                canvas.draw(&text_mesh.text, graphics::DrawParam::from([self.pos.x + 20.0, mt_y]));
            }
            GameMenuItemVariant::InputItem { text, prompt_mesh, text_highlight_rect, selected_text_highlight_rect, .. } => {
                canvas.draw(&prompt_mesh.text, graphics::DrawParam::from([self.pos.x + 20.0, self.pos.y + 10.0]));
                let pm_sz = prompt_mesh.text.measure(ctx)?;
                canvas.draw(
                    if !self.currently_selected { text_highlight_rect } else { selected_text_highlight_rect },
                    graphics::DrawParam::from([self.pos.x + 20.0, self.pos.y + pm_sz.y + 20.0]),
                );
                // Draw actual text
                let text_draw = UIText::new(
                    text.to_string(),
                    if self.currently_selected { Color::WHITE } else { Color::BLACK },
                    28.0,
                    // This doesn't really matter
                    DrawablePos { x: self.pos.x + 30.0, y: 0.0 },
                );
                let y_off = (30.0 - text_draw.text.measure(ctx)?.y) / 2.0;
                // Kind of suboptimal...
                let mut text_draw = UIText { pos: DrawablePos { y: self.pos.y + pm_sz.y + 20.0 + y_off, ..text_draw.pos }, ..text_draw };

                text_draw.draw(ctx, canvas)?;
            }
            GameMenuItemVariant::ImageItem { image, caption_mesh, scale_factor } => {
                canvas.draw(
                    image,
                    // We have to scale the image to fit in the box
                    graphics::DrawParam::from([self.pos.x + 20.0, self.pos.y + 20.0]).scale(Vec2::from((*scale_factor, *scale_factor))),
                );
                caption_mesh.draw(ctx, canvas)?;
            }
        }

        Ok(())
    }
}
