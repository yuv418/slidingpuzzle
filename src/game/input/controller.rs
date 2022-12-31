use ggez::event::{Axis, Button};

use super::InputAction;

const AXIS_TRIGGER_VALUE: f32 = 0.6;

#[derive(Default)]
pub struct GameControllerInput {
    valid_x: bool,
    valid_y: bool,

    last_x: f32,
    last_y: f32,
}

impl GameControllerInput {
    pub fn process_axis_input(&mut self, i: Axis, value: f32) -> Option<InputAction> {
        // println!("Axis input: {:?}, value {}", i, value);

        let diff_x = value - self.last_x;
        let diff_y = value - self.last_y;
        println!("value {} last_y {} diff_y {} reset {}", value, self.last_y, diff_y, (diff_y.abs() >= 0.3));
        println!("value {} last_x {} diff_x {}", value, self.last_x, diff_x);
        let r = match i {
            Axis::DPadX | Axis::RightStickX | Axis::LeftStickX if value > AXIS_TRIGGER_VALUE && !self.valid_x => {
                self.valid_x = true;
                self.last_x = value;
                println!("Triggered.");
                Some(InputAction::Right)
            }
            Axis::DPadX | Axis::RightStickX | Axis::LeftStickX if value < -AXIS_TRIGGER_VALUE && !self.valid_x => {
                self.valid_x = true;
                self.last_x = value;
                println!("Triggered.");
                Some(InputAction::Left)
            }
            Axis::RightStickX | Axis::LeftStickX if (diff_x.abs()) >= 0.6 => {
                self.valid_x = false;
                println!("Stick is now invalid, waiting for new input");
                None
            }
            Axis::DPadY | Axis::RightStickY | Axis::LeftStickY if value > AXIS_TRIGGER_VALUE && !self.valid_y => {
                self.valid_y = true;
                self.last_y = value;
                println!("Triggered.");
                Some(InputAction::Up)
            }
            Axis::DPadY | Axis::RightStickY | Axis::LeftStickY if value < -AXIS_TRIGGER_VALUE && !self.valid_y => {
                self.valid_y = true;
                self.last_y = value;
                println!("Triggered.");
                Some(InputAction::Down)
            }
            Axis::RightStickY | Axis::LeftStickY if (diff_y.abs() >= 0.6) => {
                println!("value {} last_y {} diff_y {}", value, self.last_y, diff_y);
                self.valid_y = false;
                None
            }
            _ => None,
        };
        r
    }
    pub fn process_button_input(&self, i: Button) -> Option<InputAction> {
        println!("Button input found {:?}", i);
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
