use std::sync::Arc;

use ggez::{
    glam::Vec2,
    graphics::{DrawMode, Mesh, Rect},
    input::keyboard::KeyInput,
    winit::event::VirtualKeyCode,
    Context, GameResult,
};
use keyframe::{functions::EaseInOut, keyframes, AnimationSequence};

use crate::game::{
    animation::DrawablePos,
    drawable::Drawable,
    input::InputAction,
    player::PLAYER,
    puzzle::{
        puzzle_view::PuzzleView,
        tiles::{tile_multiplayer::TileMultiplayerTransport, TileState},
    },
    resources::theme::Theme,
    scene::Scene,
    ui::uitext::UIText,
};

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
    local_user_text: UIText,
    peer_user_text: UIText,

    winner_text: UIText,
    winner_anim: AnimationSequence<f32>,
    winner: Option<Winner>,

    game_cancelled: bool,
    img_num: usize,
}

impl MultiplayerGameView {
    pub fn new(
        context: &mut Context, transport: MultiplayerTransport, img_num: usize, num_rows_cols: usize, peer_username: String,
    ) -> GameResult<Self> {
        let transport = Arc::new(transport);
        Ok(Self {
            img_num,
            user_tile_state: TileState::new(
                context,
                img_num,
                num_rows_cols,
                (0.0, 0.0),
                TileMultiplayerTransport::new(Some(transport.clone())),
                false,
            )?,
            peer_tile_state: TileState::new(
                context,
                img_num,
                num_rows_cols,
                (850.0, 0.0),
                TileMultiplayerTransport::new(Some(transport)),
                true,
            )?,
            separator_line: Mesh::new_line(
                context,
                &[Vec2::new(835.0, 0.0), Vec2::new(835.0, context.gfx.drawable_size().1)],
                10.0,
                Theme::sep_color(),
            )?,
            winner: None,
            winner_text: UIText::new(
                "Winner!".to_string(),
                Theme::fg_color(),
                88.0,
                DrawablePos {
                    // We don't know x right now.
                    x: 0.0,
                    y: 90.0,
                },
            ),
            winner_anim: keyframes![(0.0, 0.0, EaseInOut), (context.gfx.drawable_size().1, 2.0, EaseInOut)],
            local_user_text: UIText::new(
                {
                    let opt_player = PLAYER.lock().unwrap();
                    let player = opt_player.as_ref().unwrap();
                    player.username()
                },
                Theme::fg_color(),
                38.0,
                DrawablePos { x: 90.0, y: 90.0 },
            ),
            peer_user_text: UIText::new(peer_username, Theme::fg_color(), 38.0, DrawablePos { x: 90.0 + 835.0, y: 90.0 }),
            game_cancelled: false,
        })
    }
}

impl Scene for MultiplayerGameView {
    fn next_scene(&mut self, ctx: &mut Context) -> Option<Box<dyn Scene>> {
        if self.game_cancelled {
            Some(Box::new(PuzzleView::new(ctx, self.img_num).expect("Failed to create puzzle view")))
        } else {
            None
        }
    }
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        // Logic to check if there is a winner
        if let None = self.winner {
            if self.peer_tile_state.puzzle_statistics.is_some() && self.user_tile_state.puzzle_statistics.is_none() {
                // Peer won
                self.winner = Some(Winner::Peer)
            } else if self.peer_tile_state.puzzle_statistics.is_none() && self.user_tile_state.puzzle_statistics.is_some() {
                // Local won
                self.winner = Some(Winner::User)
            } else if self.peer_tile_state.puzzle_statistics.is_some() && self.user_tile_state.puzzle_statistics.is_some() {
                // Time based reconciliation
                let peer_fin_time = self.peer_tile_state.puzzle_statistics.as_ref().unwrap().finish_time;

                let user_fin_time = self.user_tile_state.puzzle_statistics.as_ref().unwrap().finish_time;
                if peer_fin_time > user_fin_time {
                    self.winner = Some(Winner::Peer)
                } else {
                    self.winner = Some(Winner::User)
                }
            }
        }

        self.peer_tile_state.update(ctx)
    }
    fn handle_input_event(&mut self, ctx: &mut Context, key_input: InputAction) {
        // This will eventually get changed
        if let InputAction::Cancel = key_input {
            self.game_cancelled = true;
        }
        self.user_tile_state.handle_input_event(ctx, key_input);
    }
}

impl Drawable for MultiplayerGameView {
    fn draw(&mut self, ctx: &mut ggez::Context, canvas: &mut ggez::graphics::Canvas) -> ggez::GameResult {
        self.user_tile_state.draw(ctx, canvas)?;
        canvas.draw(&self.separator_line, Vec2::new(0.0, 0.0));
        self.peer_tile_state.draw(ctx, canvas)?;

        // Draw usernames
        self.local_user_text.draw(ctx, canvas)?;
        self.peer_user_text.draw(ctx, canvas)?;

        if let Some(winner) = &self.winner {
            if match winner {
                Winner::User => self.user_tile_state.finished(),
                Winner::Peer => self.peer_tile_state.finished(),
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
                    Theme::bg_color(),
                )?;
                canvas.draw(&cover_rect, Vec2::new(0.0, 0.0));
                if self.winner_anim.finished() {
                    self.winner_text.pos.x = 90.0
                        + match winner {
                            Winner::User => 0.0,
                            Winner::Peer => 835.0,
                        };
                    self.winner_text.draw(ctx, canvas)?;
                }
            }
        }

        Ok(())
    }
}
