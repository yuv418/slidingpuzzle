use ggez::{
    graphics::{Canvas, DrawParam, Image},
    Context, GameResult,
};
use keyframe::{functions::EaseInOut, keyframes, AnimationSequence};
use keyframe_derive::CanTween;

use crate::game::{
    animation::animatable::{Animatable, Tweenable},
    drawable::Drawable,
};

const TILE_GAP: f32 = 20.0;

const TILE_PADDING_X: f32 = 90.0;
const TILE_PADDING_Y: f32 = 150.0;

pub struct Tile {
    // The size of a square tile (one side) in px
    pub side_len: u32,
    pub image_buf: Image,
    pub pos: TilePosition,
}

// We don't use DrawablePos because of the implementations that come after this.
#[derive(CanTween, Debug, Clone, Copy, Default)]
pub struct TilePosition {
    pub x: f32,
    pub y: f32,
    pub scale: f32,
}

impl Tweenable for TilePosition {}

impl TilePosition {
    pub fn from_ij(i: usize, j: usize, tile_size: u32, off_x: f32, off_y: f32) -> Self {
        TilePosition {
            x: off_x + TILE_PADDING_X + (j as f32 * (tile_size as f32 + TILE_GAP)),
            y: off_y + TILE_PADDING_Y + (i as f32 * (tile_size as f32 + TILE_GAP)),
            scale: 1.0,
        }
    }

    // For when the game is completed
    pub fn from_ij_no_gap(i: usize, j: usize, tile_size: u32, off_x: f32, off_y: f32) -> Self {
        TilePosition {
            x: off_x + TILE_GAP + TILE_PADDING_X + (j as f32 * (tile_size as f32)),
            y: off_y + TILE_GAP + TILE_PADDING_Y + (i as f32 * (tile_size as f32)),
            scale: 1.0,
        }
    }
}

impl Animatable<TilePosition> for Tile {
    fn set_state(&mut self, now: TilePosition) { self.pos = now; }

    fn to_state(&self, to: TilePosition, duration: f32) -> AnimationSequence<TilePosition> {
        // Keyframe all the tiles.
        if duration > 0.0 {
            keyframes![(self.pos, 0.0, EaseInOut), (to, duration, EaseInOut)]
        } else {
            keyframes![]
        }
    }
}

impl Drawable for Tile {
    fn draw(&mut self, _ctx: &mut Context, canvas: &mut Canvas) -> GameResult {
        canvas.draw(&self.image_buf, DrawParam::from([self.pos.x, self.pos.y]).scale([self.pos.scale; 2]));
        Ok(())
    }
}
