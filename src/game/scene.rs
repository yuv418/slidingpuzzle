use ggez::{input::keyboard::KeyInput, Context};

//
pub trait Scene {
    fn handle_key_event(&mut self, _ctx: &mut Context, key_input: KeyInput, repeat: bool);
}
