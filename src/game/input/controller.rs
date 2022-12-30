use ggez::event::{Axis, Button};

use super::InputAction;

const AXIS_TRIGGER_VALUE: f32 = 0.5;

#[derive(Default)]
pub struct GameControllerInput {
    valid_x: bool,
    valid_y: bool,
}

impl GameControllerInput {
    pub fn process_axis_input(&mut self, i: Axis, value: f32) -> Option<InputAction> {
        println!("Axis input: {:?}, value {}", i, value);
        match i {
            Axis::DPadX | Axis::RightStickX | Axis::LeftStickX if value > AXIS_TRIGGER_VALUE && !self.valid_x => {
                self.valid_x = true;
                Some(InputAction::Right)
            }
            Axis::DPadX | Axis::RightStickX | Axis::LeftStickX if value < -AXIS_TRIGGER_VALUE && !self.valid_x => {
                self.valid_x = true;
                Some(InputAction::Left)
            }
            Axis::DPadX | Axis::RightStickX | Axis::LeftStickX if value == 0.0 => {
                self.valid_x = false;
                None
            }
            Axis::DPadY | Axis::RightStickY | Axis::LeftStickY if value > AXIS_TRIGGER_VALUE && !self.valid_y => {
                self.valid_y = true;
                Some(InputAction::Up)
            }
            Axis::DPadY | Axis::RightStickY | Axis::LeftStickY if value < -AXIS_TRIGGER_VALUE && !self.valid_y => {
                self.valid_y = true;
                Some(InputAction::Down)
            }
            Axis::DPadY | Axis::RightStickY | Axis::LeftStickY if value == 0.0 => {
                self.valid_y = false;
                None
            }
            _ => None,
        }
    }
    pub fn process_button_input(&self, i: Button) -> Option<InputAction> {
        match i {
            Button::South => Some(InputAction::Select),
            Button::East => Some(InputAction::Cancel),
            Button::DPadDown => Some(InputAction::Down),
            Button::DPadUp => Some(InputAction::Up),
            Button::DPadRight => Some(InputAction::Right),
            Button::DPadLeft => Some(InputAction::Left),
            _ => None,
        }
    }
}
