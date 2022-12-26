use crate::game::player::PLAYER;

use super::{
    menu_item::GameMenuItem,
    menu_item_list::{GameMenuItemList, NewGameMenuItemData},
};

use crate::game::{drawable::Drawable as SlidingPuzzleDrawable, scene::Scene};
use ggez::graphics::Canvas;
use ggez::graphics::{self, Color, PxScale, Text, TextFragment};
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
    currently_selected: usize,
    to_next_scene: bool,

    title_text: Text,
    below_menu_text: Text,
}

impl GameMenu {
    pub fn new<T: GameMenuData>(ctx: &mut Context) -> GameResult<Self> {
        let title_text = Text::new(TextFragment {
            text: T::title(),
            color: Some(Color::BLACK),
            font: Some("SecularOne-Regular".into()),
            scale: Some(PxScale::from(78.0)),
        });
        let below_menu_text = Text::new(TextFragment {
            text: T::title(),
            color: Some(Color::BLACK),
            font: Some("SecularOne-Regular".into()),
            scale: Some(PxScale::from(58.0)),
        });

        let tx_s = title_text.measure(ctx)?;
        Ok(Self {
            menu_mappings: GameMenuItemList::new(
                ctx,
                T::menu_mappings(),
                90.0,
                tx_s.y + 110.0,
                tx_s.x,
                80.0,
            )?,
            title_text,
            below_menu_text,
            currently_selected: 0,
            to_next_scene: false,
        })
    }
}

impl SlidingPuzzleDrawable for GameMenu {
    fn draw(&mut self, ctx: &mut Context, canvas: &mut Canvas) -> ggez::GameResult {
        canvas.draw(
            &self.title_text,
            graphics::DrawParam::from([90.0, 90.0]).color(Color::BLACK),
        );
        self.menu_mappings.draw(ctx, canvas)?;

        canvas.draw(
            &self.below_menu_text,
            graphics::DrawParam::from([90.0, self.menu_mappings.height()]).color(Color::BLACK),
        );

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
