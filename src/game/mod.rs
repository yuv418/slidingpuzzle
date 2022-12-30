use self::input::controller::GameControllerInput;
use self::input::keyboard::KeyboardInput;
use self::resources::theme::Theme;
use self::resources::ResourceManager;
use self::scene::Scene;
use ggez::event;
use ggez::event::Axis;
use ggez::event::Button;
use ggez::glam::Vec2;
use ggez::graphics;

use ggez::event::GamepadId;
use ggez::graphics::DrawMode;
use ggez::graphics::Mesh;
use ggez::graphics::Rect;
use ggez::input::keyboard::KeyInput;
use ggez::{Context, GameResult};
use keyframe::functions::EaseInOut;
use keyframe::keyframes;
use keyframe::AnimationSequence;

pub mod animation;
pub mod drawable;
pub mod gmenu;
pub mod input;
pub mod player;
pub mod puzzle;
pub mod resources;
pub mod scene;
pub mod ui;

#[cfg(feature = "multiplayer")]
pub mod multiplayer;

pub struct GameState {
    pub current_scene: Box<dyn Scene>,
    pub prev_scene: Option<Box<dyn Scene>>,

    pub set_winsize: bool,

    // Input related
    gc_inp: GameControllerInput,

    // Scene transition animation variables
    scene_transition: Option<AnimationSequence<f32>>,
}

impl GameState {
    pub fn new(context: &mut Context) -> GameResult<Self> {
        // Loop through and make the tiles
        Ok(Self {
            current_scene: Box::new(ResourceManager::new(context)?),
            prev_scene: None,
            set_winsize: false,
            scene_transition: None,
            gc_inp: GameControllerInput::default(),
        })
    }
}

impl event::EventHandler<ggez::GameError> for GameState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        if let None = self.scene_transition {
            if let Some(next_scene) = self.current_scene.next_scene(ctx) {
                self.prev_scene = Some(std::mem::replace(&mut self.current_scene, next_scene));
                self.scene_transition =
                    Some(keyframes![(0.0, 0.0, EaseInOut), (ctx.gfx.drawable_size().1, 0.2, EaseInOut), (0.0, 0.4, EaseInOut)]);
            }
        }
        self.current_scene.update(ctx)?;
        Ok(())
    }
    fn draw(&mut self, ctx: &mut ggez::Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Theme::bg_color());

        if let Some(seq) = &mut self.scene_transition {
            seq.advance_by(0.01);
            let drawable_size = ctx.gfx.drawable_size();
            let cover_rect = Mesh::new_rectangle(
                ctx,
                DrawMode::fill(),
                Rect { x: 0.0, y: if seq.progress() > 0.5 { drawable_size.1 - seq.now() } else { 0.0 }, w: drawable_size.0, h: seq.now() },
                Theme::bg_color(),
            )?;

            if seq.progress() < 0.5 {
                self.prev_scene.as_mut().unwrap().draw_transition(ctx, &mut canvas)?;
            } else if seq.progress() == 0.5 {
                self.prev_scene = None;
            } else {
                self.current_scene.draw_transition(ctx, &mut canvas)?;
            }

            canvas.draw(&cover_rect, Vec2::new(0.0, 0.0));
            if seq.finished() {
                self.scene_transition = None;
            }
        } else {
            self.current_scene.draw(ctx, &mut canvas)?;
        }
        canvas.finish(ctx)
    }

    fn key_down_event(&mut self, ctx: &mut Context, key_input: KeyInput, repeat: bool) -> GameResult {
        if let Some(inp) = KeyboardInput::process_key_input(key_input, repeat) {
            self.current_scene.handle_input_event(ctx, inp);
        }
        Ok(())
    }

    fn text_input_event(&mut self, ctx: &mut ggez::Context, c: char) -> GameResult {
        self.current_scene.text_input_event(ctx, c);
        Ok(())
    }

    fn gamepad_button_down_event(&mut self, ctx: &mut Context, btn: Button, _id: GamepadId) -> Result<(), ggez::GameError> {
        if let Some(inp) = self.gc_inp.process_button_input(btn) {
            self.current_scene.handle_input_event(ctx, inp);
        }
        Ok(())
    }

    fn gamepad_axis_event(&mut self, ctx: &mut Context, axis: Axis, value: f32, _id: ggez::event::GamepadId) -> GameResult {
        if let Some(inp) = self.gc_inp.process_axis_input(axis, value) {
            self.current_scene.handle_input_event(ctx, inp);
        }
        Ok(())
    }
}
