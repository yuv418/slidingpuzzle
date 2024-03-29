use keyframe_derive::CanTween;

use self::animatable::Tweenable;

pub mod animatable;
pub mod animation;

#[derive(Debug, CanTween, Copy, Clone, Default)]
pub struct DrawablePos {
    pub x: f32,
    pub y: f32,
}

impl Tweenable for DrawablePos {}
