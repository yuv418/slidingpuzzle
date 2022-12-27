use ggez::{
    glam::Vec2,
    graphics::{Canvas, Image},
    Context, GameResult,
};
use keyframe::{functions::EaseInOut, keyframes, AnimationSequence};
use keyframe_derive::CanTween;

use crate::game::drawable::Drawable;

const TILE_GAP: f32 = 20.0;

const TILE_PADDING_X: f32 = 90.0;
const TILE_PADDING_Y: f32 = 150.0;

pub struct Tile {
    // The size of a square tile (one side) in px
    pub side_len: u32,
    pub image_buf: Image,
    pub pos: TilePosition,
    pub animation: Option<AnimationSequence<TilePosition>>,
}

// We don't use DrawablePos because of the implementations that come after this.
#[derive(CanTween, Debug, Clone, Copy, Default)]
pub struct TilePosition {
    pub x: f32,
    pub y: f32,
}

impl TilePosition {
    pub fn from_ij(i: usize, j: usize, tile_size: u32, off_x: f32, off_y: f32) -> Self {
        TilePosition {
            x: off_x + TILE_PADDING_X + (j as f32 * (tile_size as f32 + TILE_GAP)),
            y: off_y + TILE_PADDING_Y + (i as f32 * (tile_size as f32 + TILE_GAP)),
        }
    }

    // For when the game is completed
    pub fn from_ij_no_gap(i: usize, j: usize, tile_size: u32, off_x: f32, off_y: f32) -> Self {
        TilePosition {
            x: off_x + TILE_GAP + TILE_PADDING_X + (j as f32 * (tile_size as f32)),
            y: off_y + TILE_GAP + TILE_PADDING_Y + (i as f32 * (tile_size as f32)),
        }
    }
}

impl Tile {
    pub fn to_pos(&mut self, new_pos: TilePosition, duration: f32) {
        // Keyframe all the tiles.
        if duration > 0.0 {
            self.animation = Some(keyframes![
                (self.pos.clone(), 0.0, EaseInOut),
                (new_pos.clone(), duration, EaseInOut)
            ]);
        }
        self.pos = new_pos;
    }
}
impl Drawable for Tile {
    fn draw(&mut self, _ctx: &mut Context, canvas: &mut Canvas) -> GameResult {
        if let Some(seq) = &mut self.animation {
            seq.advance_by(0.05);
            let anim_pos = seq.now();
            canvas.draw(&self.image_buf, Vec2::new(anim_pos.x, anim_pos.y));
            if seq.finished() {
                self.animation = None;
            }
        } else {
            canvas.draw(&self.image_buf, Vec2::new(self.pos.x, self.pos.y));
        }
        Ok(())
    }
}
