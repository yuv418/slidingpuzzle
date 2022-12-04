use crate::game::player::PLAYER;

use self::menu_item::GameMenuItem;
use self::puzzle_listing::PuzzleListing;

use super::drawable::Drawable as SlidingPuzzleDrawable;
use super::player::{self, Player};
use super::scene::Scene;
use super::tiles::TileState;
use ggez::graphics::Canvas;
use ggez::graphics::{self, Color, PxScale, Text, TextFragment};
use ggez::input::keyboard::KeyInput;
use ggez::winit::event::VirtualKeyCode;
use ggez::{Context, GameResult};

pub mod menu_item;
pub mod puzzle_listing;
pub mod settings;

pub struct GameMenu {
    menu_mappings: Vec<GameMenuItem>,
    currently_selected: usize,
    to_next_scene: bool,

    title_text: Text,
}

pub fn next_page(_ctx: &mut Context) -> Box<dyn Scene> {
    println!("Going to next page");
    unimplemented!()
}
pub fn continue_game(context: &mut Context) -> Box<dyn Scene> {
    let opt_player = PLAYER.lock().unwrap();
    // Player guaranteed to be some at this point
    let player = opt_player.as_ref().unwrap();

    let tile_state = Box::new(
        TileState::new(
            context,
            if let Some(max_pzl) = player.completed_puzzles.peek() {
                // We'll have to add some kind of check to make sure
                // that the player hasn't actually completed the entire game,
                // otherwise this would cause problems.
                max_pzl + 1
            } else {
                0
            },
            player.player_settings.num_rows_cols,
        )
        .expect("Failed to create TileState"),
    );

    tile_state
}

impl GameMenu {
    pub fn new(ctx: &mut Context) -> GameResult<Self> {
        let title_text = Text::new(TextFragment {
            text: "Sliding Puzzle".to_string(),
            color: Some(Color::BLACK),
            font: Some("SecularOne-Regular".into()),
            scale: Some(PxScale::from(78.0)),
        });
        let tx_s = title_text.measure(ctx)?;
        let mut menu_mappings = vec![
            GameMenuItem::new_text_item(
                ctx,
                "Continue",
                // TODO we probably want to move continue_game to the TileState class as a
                // static method
                Box::new(continue_game),
                90.0,
                110.0 + tx_s.y,
                tx_s.x,
                80.0,
            )?,
            GameMenuItem::new_text_item(
                ctx,
                "Choose a Puzzle",
                Box::new(|context: &mut Context| {
                    Box::new(
                        PuzzleListing::new(context, 0).expect("Failed to create puzzle listing"),
                    )
                }),
                90.0,
                110.0 + tx_s.y + 100.0,
                tx_s.x,
                80.0,
            )?,
            GameMenuItem::new_text_item(
                ctx,
                "Settings",
                Box::new(next_page),
                90.0,
                110.0 + tx_s.y + 200.0,
                tx_s.x,
                80.0,
            )?,
        ];
        menu_mappings[0].select();
        Ok(Self {
            menu_mappings,
            title_text,
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

        for menu_mapping in self.menu_mappings.iter_mut() {
            menu_mapping.draw(ctx, canvas)?;
        }

        Ok(())
    }
}

impl Scene for GameMenu {
    fn handle_key_event(&mut self, _ctx: &mut Context, key_input: KeyInput, _repeat: bool) {
        if let Some(vkeycode) = key_input.keycode {
            match vkeycode {
                VirtualKeyCode::Up => {
                    if self.currently_selected > 0 {
                        self.menu_mappings[self.currently_selected].deselect();
                        self.currently_selected -= 1;
                        self.menu_mappings[self.currently_selected].select();
                    }
                }

                VirtualKeyCode::Down => {
                    if self.currently_selected < self.menu_mappings.len() - 1 {
                        self.menu_mappings[self.currently_selected].deselect();
                        self.currently_selected += 1;
                        self.menu_mappings[self.currently_selected].select();
                    }
                }
                VirtualKeyCode::Return => {
                    self.to_next_scene = true;
                }
                _ => {}
            }
        }
    }

    fn next_scene(&mut self, ctx: &mut Context) -> Option<Box<dyn Scene>> {
        if self.to_next_scene {
            Some((self.menu_mappings[self.currently_selected].next_page)(ctx))
        } else {
            None
        }
    }

    fn draw_transition(&mut self, ctx: &mut Context, canvas: &mut Canvas) -> ggez::GameResult {
        self.draw(ctx, canvas)
    }
}
