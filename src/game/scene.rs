use ggez::{input::keyboard::KeyInput, Context};

use super::drawable::Drawable;

//
pub trait Scene: Drawable {
    fn handle_key_event(&mut self, _ctx: &mut Context, key_input: KeyInput, repeat: bool);
    fn next_scene(&self, ctx: &mut Context) -> Option<Box<dyn Scene>>;
}
