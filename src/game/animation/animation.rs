use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use crate::game::animation::animatable::{Animatable, Tweenable};
use keyframe::AnimationSequence;

pub enum AnimationData<T: Tweenable> {
    Sequence((Rc<RefCell<dyn Animatable<T>>>, AnimationSequence<T>)),
    Generator((Rc<RefCell<dyn Animatable<T>>>, T, f32)),
    Simultaneous,
    Unsimultaneous,
}
impl<T: Tweenable> std::fmt::Debug for AnimationData<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AnimationData::Sequence((_, _)) => write!(f, "Sequence(_)"),
            AnimationData::Generator((_, t, i)) => write!(f, "Generator(_, {:?}, {:?}) ", t, i),
            AnimationData::Simultaneous => write!(f, "Simultaneous"),
            AnimationData::Unsimultaneous => write!(f, "Unsimultaneous"),
        }
    }
}

#[derive(Default, Debug)]
pub struct Animation<T: Tweenable> {
    animations: VecDeque<AnimationData<T>>,
    started: bool,
    simultaneous: bool,
}

impl<T: Tweenable> Animation<T> {
    pub fn new(animations: Vec<AnimationData<T>>) -> Self {
        // Initial state should be correct

        let mut modified_states = vec![];
        for i in animations.iter() {
            match i {
                AnimationData::Sequence(e) => {
                    for x in modified_states.iter() {
                        if Rc::ptr_eq(&e.0, x) {
                            continue;
                        }
                    }

                    e.0.borrow_mut().set_state(e.1.now());
                    modified_states.push(e.0.clone())
                }
                _ => {}
            }
        }

        Self { animations: VecDeque::from(animations), simultaneous: false, started: false }
    }

    // Pushes a sequence onto the animation "queue"
    pub fn push_seq(&mut self, anim: AnimationData<T>) {
        self.animations.push_back(anim);
    }

    pub fn advance(&mut self, s: f64) {
        // Treat the animations as a queue

        if !self.started {
            self.check_pop();
            self.started = true;
        }

        if !self.simultaneous {
            match self.animations.front_mut() {
                Some(AnimationData::Simultaneous) | Some(AnimationData::Unsimultaneous) => {
                    self.check_pop();
                }
                Some(AnimationData::Sequence(current_anim)) => {
                    Self::advance_individiual_animation(current_anim, s);
                    if current_anim.1.finished() {
                        self.animations.pop_front();
                        self.check_pop();
                    }
                }
                Some(AnimationData::Generator(g)) => *self.animations.front_mut().unwrap() = Self::generator_to_seq(g),
                None => {}
            }
        } else {
            // Remove any generators
            for anim in self.animations.iter_mut() {
                match anim {
                    AnimationData::Generator(gen) => *anim = Self::generator_to_seq(gen),
                    AnimationData::Unsimultaneous => break,
                    _ => {}
                }
            }

            let mut simul_anims = 0;
            let mut finished_anims = 0;
            for anim in self.animations.iter_mut() {
                match anim {
                    AnimationData::Sequence(current_anim) => {
                        Self::advance_individiual_animation(current_anim, s);
                        if current_anim.1.finished() {
                            finished_anims += 1;
                        }
                        simul_anims += 1;
                    }
                    AnimationData::Unsimultaneous => break,
                    _ => {}
                }
            }

            if simul_anims == finished_anims {
                for _ in 0..simul_anims {
                    self.animations.pop_front();
                }
                self.check_pop();
            }
        }
    }

    fn generator_to_seq((animatable, final_state, duration): &(Rc<RefCell<dyn Animatable<T>>>, T, f32)) -> AnimationData<T> {
        let keyframes = animatable.borrow_mut().to_state(*final_state, *duration);
        AnimationData::Sequence((animatable.clone(), keyframes))
    }

    fn check_pop(&mut self) {
        // Pop the next element if it's not an animation sequence
        let ins_item = match self.animations.front_mut() {
            Some(AnimationData::Simultaneous) | Some(AnimationData::Unsimultaneous) => {
                match self.animations.front() {
                    Some(AnimationData::Simultaneous) => self.simultaneous = true,
                    Some(AnimationData::Unsimultaneous) => self.simultaneous = false,
                    _ => {}
                }
                println!("set simultaneous to {:?}", self.simultaneous);
                self.animations.pop_front();
                None
            }
            Some(AnimationData::Generator(gen)) => {
                let i = Self::generator_to_seq(gen);
                self.animations.pop_front();
                Some(i)
            }
            _ => None,
        };
        if let Some(item) = ins_item {
            self.animations.push_front(item);
        }
    }

    fn advance_individiual_animation(anim: &mut (Rc<RefCell<dyn Animatable<T>>>, AnimationSequence<T>), s: f64) {
        anim.1.advance_by(s);
        anim.0.borrow_mut().set_state(anim.1.now());
    }

    pub fn finished(&self) -> bool {
        self.animations.is_empty()
    }
}
