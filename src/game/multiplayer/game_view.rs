use std::sync::Arc;

use ggez::{
    glam::Vec2,
    graphics::{Color, DrawParam, Mesh},
    input::keyboard::KeyInput,
    Context, GameResult,
};

use crate::game::{drawable::Drawable, scene::Scene, tiles::TileState};

use super::transport::MultiplayerTransport;

pub struct MultiplayerGameView {
    user_tile_state: TileState,
    peer_tile_state: TileState,
    // Meshes
    separator_line: Mesh,
}

impl MultiplayerGameView {
    pub fn new(
        context: &mut Context,
        transport: MultiplayerTransport,
        img_num: usize,
        num_rows_cols: usize,
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
                    Vec2::new(815.0, 0.0),
                    Vec2::new(815.0, context.gfx.drawable_size().1),
                ],
                10.0,
                Color::RED,
            )?,
        })
    }
}

impl Scene for MultiplayerGameView {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
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
        Ok(())
    }
}
