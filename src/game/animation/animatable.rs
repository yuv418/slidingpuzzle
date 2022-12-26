use keyframe::CanTween;

pub trait Tweenable: CanTween + Default + Copy {}

pub trait Animatable<T>
where
    T: CanTween + Copy + Default,
{
    // Set animation state of item
    fn set_state(&mut self, now: T);
}
