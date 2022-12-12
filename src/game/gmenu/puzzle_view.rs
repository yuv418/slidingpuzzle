// Display a puzzle, choose singleplayer or multiplayer
//
//
// Multiplayer host workflow
// -> Choose multiplayer
// -> Display string + copy to clipboard
// -> Start game
//
// Multiplayer client workflow
// -> Join multiplayer
// -> Paste connection string
// -> Display string + copy to clipboard
// -> Start game

use ggez::{
    glam::Vec2,
    graphics::{Color, DrawParam, Image, PxScale, Text, TextFragment},
    input::keyboard::KeyInput,
    winit::event::VirtualKeyCode,
    Context, GameResult,
};

use crate::game::{
    drawable::Drawable,
    multiplayer::{self, join_scene::JoinMultiplayerScene},
    player::PLAYER,
    scene::Scene,
    tiles::TileState,
};

use super::{menu_item::GameMenuItem, puzzle_listing::PuzzleListing};

pub struct PuzzleView {
    title_text: Text,

    // Image
    puzzle_image: Image,

    selected_action: usize,
    puzzle_action_mappings: Vec<GameMenuItem>,
    puzzle_num: usize,

    next_page: bool,
    back: bool,
}

impl PuzzleView {
    pub fn new(ctx: &mut Context, puzzle_num: usize) -> GameResult<Self> {
        let mut puzzle_action_mappings = vec![
            GameMenuItem::new_text_item(
                ctx,
                "Play as Singleplayer",
                // TODO we probably want to move continue_game to the TileState class as a
                // static method
                Box::new(move |context: &mut Context| {
                    let opt_player = PLAYER.lock().unwrap();
                    let player = opt_player.as_ref().unwrap();
                    Box::new(
                        TileState::new(
                            context,
                            puzzle_num,
                            player.player_settings.num_rows_cols,
                            0.0,
                            0.0,
                            None,
                            false,
                        )
                        .expect("Failed to create singleplayer game"),
                    )
                }),
                90.0,
                520.0,
                500.0,
                80.0,
            )?,
            GameMenuItem::new_text_item(
                ctx,
                "Create Multiplayer Game",
                Box::new(move |context: &mut Context| {
                    Box::new(
                        JoinMultiplayerScene::new(context, puzzle_num, true)
                            .expect("Failed to create join multiplayer scene"),
                    )
                }),
                90.0,
                630.0,
                500.0,
                80.0,
            )?,
        ];
        puzzle_action_mappings[0].select();
        Ok(Self {
            title_text: Text::new(TextFragment {
                text: format!("Puzzle {}", puzzle_num + 1),
                color: Some(Color::BLACK),
                font: Some("SecularOne-Regular".into()),
                scale: Some(PxScale::from(78.0)),
            }),
            back: false,
            puzzle_num,
            puzzle_image: Image::from_path(ctx, format!("/images/{}.jpg", puzzle_num))?,
            selected_action: 0,
            puzzle_action_mappings,
            next_page: false,
        })
    }
}

impl Drawable for PuzzleView {
    fn draw(&mut self, ctx: &mut Context, canvas: &mut ggez::graphics::Canvas) -> GameResult {
        let scale_factor = 300.0 / self.puzzle_image.width() as f32;
        let text_dim = self.title_text.measure(ctx)?;
        canvas.draw(
            &self.puzzle_image,
            DrawParam::from([90.0, 90.0 + text_dim.y + 20.0]).scale([scale_factor; 2]),
        );
        for item in &mut self.puzzle_action_mappings {
            item.draw(ctx, canvas)?;
        }
        canvas.draw(&self.title_text, Vec2::new(90.0, 90.0));
        Ok(())
    }
}

impl Scene for PuzzleView {
    fn handle_key_event(&mut self, ctx: &mut Context, key_input: KeyInput, repeat: bool) {
        let mut old_selected = self.selected_action;
        if let Some(vkeycode) = key_input.keycode {
            match vkeycode {
                VirtualKeyCode::Up => {
                    if self.selected_action > 0 {
                        self.selected_action -= 1
                    }
                }
                VirtualKeyCode::Down => {
                    if self.selected_action + 1 < self.puzzle_action_mappings.len() {
                        self.selected_action += 1
                    }
                }
                VirtualKeyCode::Return => {
                    self.next_page = true;
                }
                VirtualKeyCode::Escape => self.back = true,
                _ => {}
            }
        }
        if old_selected != self.selected_action {
            self.puzzle_action_mappings[old_selected].deselect();
            self.puzzle_action_mappings[self.selected_action].select();
        }
    }

    fn next_scene(&mut self, ctx: &mut Context) -> Option<Box<dyn Scene>> {
        if self.next_page {
            Some((self.puzzle_action_mappings[self.selected_action].next_page)(ctx))
        } else if self.back {
            Some(Box::new(
                PuzzleListing::new(ctx, self.puzzle_num / 4)
                    .expect("Failed to return to puzzle listing"),
            ))
        } else {
            None
        }
    }
}
