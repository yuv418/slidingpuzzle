use std::{cell::RefCell, rc::Rc};

use crate::game::animation::animatable::{Animatable, Tweenable};
use keyframe::AnimationSequence;

pub struct Animation<T: Tweenable> {
    animation: Vec<(Rc<RefCell<dyn Animatable<T>>>, AnimationSequence<T>)>,
}

impl<T: Tweenable> Animation<T> {
    pub fn new(animation: Vec<(Rc<RefCell<dyn Animatable<T>>>, AnimationSequence<T>)>) -> Self {
        // Initial state should be correct
        for i in animation.iter() {
            i.0.borrow_mut().set_state(i.1.now());
        }
        Self { animation }
    }

    pub fn advance(&mut self, s: f64) {
        // Treat the animations as a queue
        if let Some(current_anim) = self.animation.last_mut() {
            current_anim.1.advance_by(s);
            current_anim.0.borrow_mut().set_state(current_anim.1.now());

            if current_anim.1.finished() {
                self.animation.pop();
            }
        }
    }

    pub fn finished(&self) -> bool {
        self.animation.is_empty()
    }
}
