use keyframe::{keyframes, AnimationSequence, CanTween};

pub trait Tweenable: std::fmt::Debug + CanTween + Default + Copy {}

pub trait Animatable<T>
where
    T: CanTween + Copy + Default,
{
    // Set animation state of item
    fn set_state(&mut self, now: T);
    fn to_state(&self, _to: T, _duration: f32) -> AnimationSequence<T> { keyframes![] }
}
