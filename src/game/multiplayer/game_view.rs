use std::sync::Arc;

use ggez::{
    glam::Vec2,
    graphics::{Color, DrawMode, DrawParam, Mesh, PxScale, Rect, Text, TextFragment},
    input::keyboard::KeyInput,
    Context, GameResult,
};
use keyframe::{functions::EaseInOut, keyframes, AnimationSequence};

use crate::game::{drawable::Drawable, player::PLAYER, scene::Scene, tiles::TileState};

use super::transport::MultiplayerTransport;

pub enum Winner {
    User,
    Peer,
}

pub struct MultiplayerGameView {
    user_tile_state: TileState,
    peer_tile_state: TileState,
    // Meshes
    separator_line: Mesh,

    // Username display
    local_user_text: Text,
    peer_user_text: Text,

    winner_text: Text,
    winner_anim: AnimationSequence<f32>,
    winner: Option<Winner>,
}

impl MultiplayerGameView {
    pub fn new(
        context: &mut Context,
        transport: MultiplayerTransport,
        img_num: usize,
        num_rows_cols: usize,
        peer_username: String,
    ) -> GameResult<Self> {
        let transport = Arc::new(transport);
        Ok(Self {
            user_tile_state: TileState::new(
                context,
                img_num,
                num_rows_cols,
                0.0,
                0.0,
                Some(transport.clone()),
                false,
            )?,
            peer_tile_state: TileState::new(
                context,
                img_num,
                num_rows_cols,
                850.0,
                0.0,
                Some(transport),
                true,
            )?,
            separator_line: Mesh::new_line(
                context,
                &[
                    Vec2::new(835.0, 0.0),
                    Vec2::new(835.0, context.gfx.drawable_size().1),
                ],
                10.0,
                Color::RED,
            )?,
            winner: None,
            winner_text: Text::new(TextFragment {
                text: "Winner!".to_string(),
                color: Some(Color::BLACK),
                font: Some("SecularOne-Regular".into()),
                scale: Some(PxScale::from(88.0)),
            }),
            winner_anim: keyframes![
                (0.0, 0.0, EaseInOut),
                (context.gfx.drawable_size().1, 2.0, EaseInOut)
            ],
            local_user_text: Text::new(TextFragment {
                text: {
                    let opt_player = PLAYER.lock().unwrap();
                    let player = opt_player.as_ref().unwrap();
                    player.username()
                },
                color: Some(Color::BLACK),
                font: Some("SecularOne-Regular".into()),
                scale: Some(PxScale::from(38.0)),
            }),
            peer_user_text: Text::new(TextFragment {
                text: peer_username,
                color: Some(Color::BLACK),
                font: Some("SecularOne-Regular".into()),
                scale: Some(PxScale::from(38.0)),
            }),
        })
    }
}

impl Scene for MultiplayerGameView {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        // Logic to check if there is a winner
        if let None = self.winner {
            if self.peer_tile_state.puzzle_statistics.is_some()
                && self.user_tile_state.puzzle_statistics.is_none()
            {
                // Peer won
                self.winner = Some(Winner::Peer)
            } else if self.peer_tile_state.puzzle_statistics.is_none()
                && self.user_tile_state.puzzle_statistics.is_some()
            {
                // Local won
                self.winner = Some(Winner::User)
            } else if self.peer_tile_state.puzzle_statistics.is_some()
                && self.user_tile_state.puzzle_statistics.is_some()
            {
                // Time based reconciliation
                let peer_fin_time = self
                    .peer_tile_state
                    .puzzle_statistics
                    .as_ref()
                    .unwrap()
                    .finish_time;

                let user_fin_time = self
                    .user_tile_state
                    .puzzle_statistics
                    .as_ref()
                    .unwrap()
                    .finish_time;
                if peer_fin_time > user_fin_time {
                    self.winner = Some(Winner::Peer)
                } else {
                    self.winner = Some(Winner::User)
                }
            }
        }

        self.peer_tile_state.update(ctx)
    }
    fn handle_key_event(&mut self, ctx: &mut Context, key_input: KeyInput, repeat: bool) {
        self.user_tile_state
            .handle_key_event(ctx, key_input, repeat);
    }
}

impl Drawable for MultiplayerGameView {
    fn draw(
        &mut self,
        ctx: &mut ggez::Context,
        canvas: &mut ggez::graphics::Canvas,
    ) -> ggez::GameResult {
        self.user_tile_state.draw(ctx, canvas)?;
        canvas.draw(&self.separator_line, Vec2::new(0.0, 0.0));
        self.peer_tile_state.draw(ctx, canvas)?;

        // Draw usernames
        canvas.draw(&self.local_user_text, Vec2::new(90.0, 90.0));
        canvas.draw(&self.peer_user_text, Vec2::new(90.0 + 835.0, 90.0));

        if let Some(winner) = &self.winner {
            if match winner {
                Winner::User => self.user_tile_state.current_animation.is_none(),
                Winner::Peer => self.peer_tile_state.current_animation.is_none(),
            } {
                if !self.winner_anim.finished() {
                    self.winner_anim.advance_by(0.05);
                }
                let cover_rect = Mesh::new_rectangle(
                    ctx,
                    DrawMode::fill(),
                    Rect {
                        x: match winner {
                            Winner::User => 0.0,
                            Winner::Peer => 835.0,
                        },
                        y: 0.0,
                        w: 800.0,
                        h: self.winner_anim.now(),
                    },
                    Color::WHITE,
                )?;
                canvas.draw(&cover_rect, Vec2::new(0.0, 0.0));
                if self.winner_anim.finished() {
                    canvas.draw(
                        &self.winner_text,
                        Vec2::new(
                            90.0 + match winner {
                                Winner::User => 0.0,
                                Winner::Peer => 835.0,
                            },
                            90.0,
                        ),
                    );
                }
            }
        }

        Ok(())
    }
}
