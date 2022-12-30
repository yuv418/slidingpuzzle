use ggez::input::keyboard::KeyInput;

use super::InputAction;

pub struct KeyboardInput {}
impl KeyboardInput {
    pub fn process_key_input(i: KeyInput, repeat: bool) -> Option<InputAction> {
        let vkeycode = i.keycode;
        if let Some(vkeycode) = vkeycode {
            if !repeat {
                return match vkeycode {
                    ggez::winit::event::VirtualKeyCode::Escape => Some(InputAction::Cancel),
                    ggez::winit::event::VirtualKeyCode::Left => Some(InputAction::Left),
                    ggez::winit::event::VirtualKeyCode::Up => Some(InputAction::Up),
                    ggez::winit::event::VirtualKeyCode::Right => Some(InputAction::Right),
                    ggez::winit::event::VirtualKeyCode::Down => Some(InputAction::Down),
                    ggez::winit::event::VirtualKeyCode::Return => Some(InputAction::Select),
                    _ => None,
                };
            }
        }
        None
    }
}
