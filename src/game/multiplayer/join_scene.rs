// Join existing game:
// Prompt + Wait for clipboard
// Display conn string + copy to clipboard

// Create game
// Display conn string + copy clipboard + wait for clipboard

use ggez::{
    glam::Vec2,
    graphics::{Color, PxScale, Text, TextFragment},
    Context, GameResult,
};

use crate::game::{drawable::Drawable, scene::Scene};

use super::{transport::MultiplayerTransport, MultiplayerGameMessage};

pub struct JoinMultiplayerScene {
    connecting: bool,
    wait_for_clipboard: Text,
    header: Text,
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
            header: Text::new(TextFragment {
                text: if conn_str.is_none() {
                    "Create Multiplayer Game"
                } else {
                    "Join Multiplayer Game"
                }
                .to_string(),
                color: Some(Color::BLACK),
                font: Some("SecularOne-Regular".into()),
                scale: Some(PxScale::from(58.0)),
            }),
            conn_string: None,
            puzzle_num,
            transport: MultiplayerTransport::create_game(conn_str)?,
        })
    }
}

impl Drawable for JoinMultiplayerScene {
    fn draw(&mut self, ctx: &mut ggez::Context, canvas: &mut ggez::graphics::Canvas) -> GameResult {
        canvas.draw(&self.header, Vec2::from((90.0, 90.0)));
        if let Some(conn_str) = &self.conn_string {
            canvas.draw(conn_str, Vec2::from((90.0, 90.0)));
        }
        if self.connecting {
            canvas.draw(&self.wait_for_clipboard, Vec2::from((90.0, 90.0)));
        }
        Ok(())
    }
}

impl Scene for JoinMultiplayerScene {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        if let Ok(event) = self.transport.event_buffer.try_recv() {
            match event {
                MultiplayerGameMessage::ConnectionString(s) => {
                    self.conn_string = Some(Text::new(TextFragment {
                        text: s,
                        color: Some(Color::BLACK),
                        font: Some("SecularOne-Regular".into()),
                        scale: Some(PxScale::from(38.0)),
                    }))
                }
            }
        }
        Ok(())
    }
}
