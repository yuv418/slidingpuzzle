// Join existing game:
// Prompt + Wait for clipboard
// Display conn string + copy to clipboard

// Create game
// Display conn string + copy clipboard + wait for clipboard

use arboard::Clipboard;
use ggez::{winit::event::VirtualKeyCode, Context, GameError, GameResult};
use log::info;

use crate::game::{
    animation::DrawablePos, drawable::Drawable, player::PLAYER, puzzle::puzzle_view::PuzzleView, resources::theme::Theme, scene::Scene,
    ui::uitext::UIText,
};

use super::{game_view::MultiplayerGameView, transport::MultiplayerTransport, MultiplayerGameMessage};

pub struct JoinMultiplayerScene {
    connecting: bool,
    creator: bool,
    wait_for_clipboard: UIText,
    header: UIText,
    conn_string: Option<UIText>,
    transport: Option<MultiplayerTransport>,
    clipboard: Clipboard,
    puzzle_num: usize,
    game_cancelled: bool,
    game_started: Option<MultiplayerGameMessage>,

    peer_username: Option<String>,
}

impl JoinMultiplayerScene {
    pub fn new(_ctx: &mut Context, puzzle_num: usize, creator: bool) -> GameResult<Self> {
        let header = UIText::new(
            if creator { "Create Multiplayer Game" } else { "Join Multiplayer Game" }.to_string(),
            Theme::fg_color(),
            58.0,
            DrawablePos { x: 90.0, y: 90.0 },
        );

        Ok(JoinMultiplayerScene {
            connecting: !creator,
            creator,
            wait_for_clipboard: UIText::new(
                "Press Enter when you have copied\nthe other player's connection string.".to_string(),
                Theme::fg_color(),
                38.0,
                DrawablePos { x: 90.0, y: 0.0 },
            ),
            header,
            conn_string: None,
            transport: if creator { Some(MultiplayerTransport::create_game(None)?) } else { None },
            clipboard: Clipboard::new().map_err(|_| GameError::CustomError("Failed to get game clipboard".to_string()))?,
            puzzle_num,
            game_cancelled: false,
            game_started: None,
            peer_username: None,
        })
    }
}

impl Drawable for JoinMultiplayerScene {
    fn draw(&mut self, ctx: &mut ggez::Context, canvas: &mut ggez::graphics::Canvas) -> GameResult {
        self.header.draw(ctx, canvas)?;
        if let Some(conn_str) = &mut self.conn_string {
            conn_str.draw(ctx, canvas)?;
        }
        if self.connecting {
            self.wait_for_clipboard.pos.y = self.header.text.measure(ctx)?.y
                + if let Some(cs) = &self.conn_string { cs.text.measure(ctx)?.y } else { 0.0 }
                + 10.0
                + 90.0;
            self.wait_for_clipboard.draw(ctx, canvas)?;
        }
        Ok(())
    }
}

impl Scene for JoinMultiplayerScene {
    fn next_scene(&mut self, ctx: &mut Context) -> Option<Box<dyn Scene>> {
        if let Some(MultiplayerGameMessage::StartGame { img_num, num_rows_cols, host_username }) = &self.game_started {
            let transport = self.transport.take().unwrap();
            Some(Box::new(
                MultiplayerGameView::new(
                    ctx,
                    transport,
                    *img_num,
                    *num_rows_cols,
                    if self.creator { self.peer_username.take().unwrap() } else { host_username.clone() },
                )
                .expect("Failed to create multiplayer game view"),
            ))
        } else if self.game_cancelled {
            Some(Box::new(PuzzleView::new(ctx, self.puzzle_num).expect("Failed to return to puzzle listing")))
        } else {
            None
        }
    }

    fn update(&mut self, ctx: &mut Context) -> GameResult {
        if let Some(transport) = &self.transport {
            if let Ok(event) = transport.event_buffer.try_recv() {
                match event {
                    MultiplayerGameMessage::ConnectionString(s) => {
                        self.clipboard
                            .set_text(&s)
                            .map_err(|_| GameError::CustomError("Failed to copy connection string to clipboard".to_string()))?;
                        self.conn_string = Some(UIText::new(
                            "Copied connection string to clipboard!".to_string(),
                            Theme::fg_color(),
                            48.0,
                            DrawablePos { x: 90.0, y: self.header.text.measure(ctx)?.y + 90.0 },
                        ));
                        self.connecting = true;
                    }
                    MultiplayerGameMessage::Hello { username } => {
                        info!("Hello recv, starting game");
                        let opt_player = PLAYER.lock().unwrap();
                        let player = opt_player.as_ref().unwrap();

                        self.peer_username = Some(username);

                        self.game_started = Some(MultiplayerGameMessage::StartGame {
                            img_num: self.puzzle_num,
                            num_rows_cols: player.player_settings.num_rows_cols,
                            host_username: player.username(),
                        });
                        // TODO handle this better
                        self.transport
                            .as_ref()
                            .unwrap()
                            .event_push_buffer
                            .send(self.game_started.as_ref().unwrap().clone())
                            .expect("Failed to send game start event to peer");
                    }
                    MultiplayerGameMessage::StartGame { .. } => {
                        self.game_started = Some(event.clone());
                    }
                    _ => {
                        println!("recv from channel {:?}", event)
                    }
                }
            }
        }
        Ok(())
    }
    fn handle_key_event(&mut self, _ctx: &mut Context, key_input: ggez::input::keyboard::KeyInput, _repeat: bool) {
        if let Some(vkeycode) = key_input.keycode {
            match vkeycode {
                VirtualKeyCode::Escape => {
                    self.game_cancelled = true;
                }
                VirtualKeyCode::Return => {
                    let conn_str = self
                        .clipboard
                        .get_text()
                        .map_err(|_| GameError::CustomError("Failed to get connection string from clipboard".to_string()))
                        // Make this method return a result so this doesn't happen
                        .unwrap();
                    if self.creator {
                        if let Err(_e) =
                            self.transport.as_ref().unwrap().event_push_buffer.send(MultiplayerGameMessage::ConnectionString(conn_str))
                        {
                            println!("failed to send event");
                        }
                    } else {
                        self.transport = Some(MultiplayerTransport::create_game(Some(conn_str)).unwrap());

                        self.transport
                            .as_ref()
                            .unwrap()
                            .event_push_buffer
                            .send(MultiplayerGameMessage::Hello {
                                username: {
                                    let opt_p = PLAYER.lock().unwrap();
                                    let p = opt_p.as_ref().unwrap();
                                    p.username()
                                },
                            })
                            .unwrap();
                    }
                }
                _ => {}
            }
        }
    }
}
