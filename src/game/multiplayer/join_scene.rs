// Join existing game:
// Prompt + Wait for clipboard
// Display conn string + copy to clipboard

// Create game
// Display conn string + copy clipboard + wait for clipboard

use ggez::{
    graphics::{Color, PxScale, Text, TextFragment},
    Context, GameResult,
};

use crate::game::{drawable::Drawable, scene::Scene};

use super::transport::MultiplayerTransport;

pub struct JoinMultiplayerScene {
    connecting: bool,
    wait_for_clipboard: Text,
    conn_string: Option<Text>,
    transport: MultiplayerTransport,
    puzzle_num: usize,
}

impl JoinMultiplayerScene {
    pub fn new(ctx: &mut Context, puzzle_num: usize, conn_str: Option<String>) -> GameResult<Self> {
        println!("new multiplayer scene");
        Ok(JoinMultiplayerScene {
            connecting: conn_str.is_some(),
            wait_for_clipboard: Text::new(TextFragment {
                text: "Press Enter when you have copied the other player's connection string."
                    .to_string(),
                color: Some(Color::BLACK),
                font: Some("SecularOne-Regular".into()),
                scale: Some(PxScale::from(38.0)),
            }),
            conn_string: None,
            puzzle_num,
            transport: MultiplayerTransport::create_game(conn_str)?,
        })
    }
}

impl Drawable for JoinMultiplayerScene {
    fn draw(&mut self, ctx: &mut ggez::Context, canvas: &mut ggez::graphics::Canvas) -> GameResult {
        todo!()
    }
}

impl Scene for JoinMultiplayerScene {}
