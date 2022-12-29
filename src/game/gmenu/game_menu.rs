use crate::game::resources::theme::Theme;
use crate::game::{animation::DrawablePos, ui::uitext::UIText};

use super::menu_item_list::{GameMenuItemList, NewGameMenuItemData};

use crate::game::{drawable::Drawable as SlidingPuzzleDrawable, scene::Scene};
use ggez::graphics::Canvas;

use ggez::input::keyboard::KeyInput;
use ggez::{Context, GameResult};

pub trait GameMenuData {
    fn menu_mappings() -> Vec<NewGameMenuItemData>;
    fn title() -> String;
    fn below_menu_text() -> Option<String> {
        None
    }
}

pub struct GameMenu {
    menu_mappings: GameMenuItemList,
    title_text: UIText,
}

impl GameMenu {
    pub fn new<T: GameMenuData>(ctx: &mut Context) -> GameResult<Self> {
        let title_text = UIText::new(T::title(), Theme::fg_color(), 78.0, DrawablePos { x: 90.0, y: 90.0 });

        let tx_s = title_text.text.measure(ctx)?;
        Ok(Self { menu_mappings: GameMenuItemList::new(ctx, T::menu_mappings(), 90.0, tx_s.y + 110.0, tx_s.x, 80.0)?, title_text })
    }
}

impl SlidingPuzzleDrawable for GameMenu {
    fn draw(&mut self, ctx: &mut Context, canvas: &mut Canvas) -> ggez::GameResult {
        self.title_text.draw(ctx, canvas)?;
        self.menu_mappings.draw(ctx, canvas)?;

        Ok(())
    }
}

impl Scene for GameMenu {
    fn handle_key_event(&mut self, ctx: &mut Context, key_input: KeyInput, repeat: bool) {
        self.menu_mappings.handle_key_event(ctx, key_input, repeat);
    }

    fn next_scene(&mut self, ctx: &mut Context) -> Option<Box<dyn Scene>> {
        self.menu_mappings.next_scene(ctx)
    }
}
