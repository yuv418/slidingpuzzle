use ggez::{graphics::Image, input::keyboard::KeyInput, winit::event::VirtualKeyCode, Context, GameResult};

use crate::game::{
    animation::{animatable::Animatable, DrawablePos},
    drawable::Drawable,
    scene::Scene,
};

use super::menu_item::GameMenuItem;

pub enum NewGameMenuItemDataVariant {
    TextItem { text: String },
    InputItem { is_num: bool, prompt: String, initial_value: String },
    ImageItem { image: Image, caption: String },
}
pub struct NewGameMenuItemData {
    pub variant: NewGameMenuItemDataVariant,
    pub next_page: Option<Box<dyn Fn(&mut Context) -> Box<dyn Scene>>>,
}

pub struct GameMenuItemList {
    pub items: Vec<GameMenuItem>,
    selected_item: usize,
    has_next_scene: bool,
    w: f32,
    h: f32,
}

const MENU_ITEM_GAP: f32 = 30.0;

impl GameMenuItemList {
    pub fn new(ctx: &mut Context, variant_items: Vec<NewGameMenuItemData>, x: f32, start_y: f32, w: f32, h: f32) -> GameResult<Self> {
        let mut items = variant_items
            .into_iter()
            .enumerate()
            .map(|(i, e)| {
                let y = start_y + ((MENU_ITEM_GAP + h) * i as f32);
                match e.variant {
                    NewGameMenuItemDataVariant::TextItem { text } => GameMenuItem::new_text_item(ctx, &text, e.next_page, x, y, w, h),
                    NewGameMenuItemDataVariant::InputItem { is_num, prompt, initial_value } => {
                        GameMenuItem::new_input_item(ctx, &prompt, initial_value.to_string(), is_num, e.next_page, x, y, w, h)
                    }
                    NewGameMenuItemDataVariant::ImageItem { image, caption } => {
                        GameMenuItem::new_image_item(ctx, image, &caption, e.next_page, x, y, w, h)
                    }
                }
            })
            .collect::<Result<Vec<_>, _>>()?;

        let selected_item = 0;
        items[selected_item].select();

        Ok(Self { items, selected_item, w, h, has_next_scene: false })
    }

    pub fn height(&self) -> f32 {
        (self.h + MENU_ITEM_GAP) * self.items.len() as f32 - MENU_ITEM_GAP
    }
}

impl Drawable for GameMenuItemList {
    fn draw(&mut self, ctx: &mut Context, canvas: &mut ggez::graphics::Canvas) -> GameResult {
        for menu_item in &mut self.items {
            menu_item.draw(ctx, canvas)?;
        }

        Ok(())
    }
}

impl Scene for GameMenuItemList {
    fn handle_key_event(&mut self, _ctx: &mut Context, key_input: KeyInput, _: bool) {
        if let Some(vkeycode) = key_input.keycode {
            match vkeycode {
                VirtualKeyCode::Up => {
                    if self.selected_item > 0 {
                        self.items[self.selected_item].deselect();
                        self.selected_item -= 1;
                        self.items[self.selected_item].select();
                    }
                }

                VirtualKeyCode::Down => {
                    if self.selected_item < self.items.len() - 1 {
                        self.items[self.selected_item].deselect();
                        self.selected_item += 1;
                        self.items[self.selected_item].select();
                    }
                }
                VirtualKeyCode::Return => {
                    self.has_next_scene = true;
                }
                _ => {}
            }
        }
    }

    fn next_scene(&mut self, ctx: &mut Context) -> Option<Box<dyn Scene>> {
        if self.has_next_scene {
            if let Some(next_page) = &self.items[self.selected_item].next_page {
                Some((next_page)(ctx))
            } else {
                None
            }
        } else {
            None
        }
    }

    fn text_input_event(&mut self, _ctx: &mut Context, c: char) {
        // Will always be the case (for now). We don't want spaces.
        // This is not the best solution, but it is a solution.
        if c != ' ' {
            self.items[self.selected_item].handle_input(c)
        }
    }
}

impl Animatable<DrawablePos> for GameMenuItemList {
    fn set_state(&mut self, now: DrawablePos) {
        for (i, mut e) in self.items.iter_mut().enumerate() {
            e.pos.y = now.y + ((MENU_ITEM_GAP + self.h) * i as f32);
            e.pos.x = now.x;
        }
    }
}
