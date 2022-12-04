use super::drawable::Drawable as SlidingPuzzleDrawable;
use super::player;
use super::scene::Scene;
use super::tiles::TileState;
use ggez::glam::Vec2;
use ggez::graphics::Canvas;
use ggez::graphics::{self, Color, DrawMode, Mesh, PxScale, Rect, Text, TextFragment};
use ggez::input::keyboard::KeyInput;
use ggez::winit::event::VirtualKeyCode;
use ggez::Context;
use keyframe::functions::EaseInOut;
use keyframe::{keyframes, AnimationSequence};

pub mod puzzle_listing;
pub mod settings;

pub struct GameMenu {
    menu_mappings: Vec<GameMenuMapping>,
    currently_selected: usize,
    to_next_scene: bool,

    // Animation stuff
    should_animate_from: Option<usize>,
    in_animating_selection: Option<AnimationSequence<f32>>,
    out_animating_selection: Option<AnimationSequence<f32>>,
    animation_finished: bool,
}

// We use a struct since it might help later for stuff such as colours
pub struct GameMenuMapping {
    pub text: String,
    pub next_page: Box<dyn Fn(&mut Context) -> Box<dyn Scene>>,
}

pub fn next_page(ctx: &mut Context) -> Box<dyn Scene> {
    println!("Going to next page");
    unimplemented!()
}

impl GameMenu {
    pub fn new(ctx: &mut Context) -> Self {
        let menu_mappings = vec![
            GameMenuMapping {
                text: "Continue".to_string(),
                next_page: Box::new(|context: &mut Context| {
                    let player = player::Player::load(context).expect("Failed to load player");
                    let tile_state = Box::new(
                        TileState::new(
                            context,
                            player.completed_puzzles,
                            player.player_settings.num_rows_cols,
                            0.0,
                            0.0,
                        )
                        .expect("Failed to create TileState"),
                    );

                    let tile_gap = tile_state.tiles[0][0].borrow().side_len + 10; // determine the gap here
                    let win_width = (180 + (tile_state.tiles[0].len() as u32 * tile_gap)) as f32;
                    let win_height = (300 + (tile_gap * tile_state.tiles.len() as u32)) as f32;
                    println!("the new window dimensions are {}x{}", win_width, win_height);
                    context.gfx.set_mode(
                        ggez::conf::WindowMode::default()
                            .dimensions(win_width, win_height)
                            .resizable(true),
                    );
                    tile_state
                }),
            },
            GameMenuMapping {
                text: "Choose a Puzzle".to_string(),
                next_page: Box::new(next_page),
            },
            GameMenuMapping {
                text: "Settings".to_string(),
                next_page: Box::new(next_page),
            },
        ];
        Self {
            menu_mappings,
            currently_selected: 0,
            to_next_scene: false,
            should_animate_from: None,
            in_animating_selection: None,
            out_animating_selection: None,
            animation_finished: true,
        }
    }
}

impl SlidingPuzzleDrawable for GameMenu {
    fn draw(&mut self, ctx: &mut Context, canvas: &mut Canvas) -> ggez::GameResult {
        let title = Text::new(TextFragment {
            text: "Sliding Puzzle".to_string(),
            color: Some(Color::BLACK),
            font: Some("SecularOne-Regular".into()),
            scale: Some(PxScale::from(78.0)),
        });
        canvas.draw(
            &title,
            graphics::DrawParam::from([90.0, 90.0]).color(Color::BLACK),
        );

        let t_sz = title
            .measure(ctx)
            .expect("Failed to measure size of title text");

        for (i, menu_mapping) in self.menu_mappings.iter().enumerate() {
            let label_box_y = (i as f32 * 110.0) + t_sz.y + 110.0;
            let label_box = Mesh::new_rounded_rectangle(
                ctx,
                DrawMode::fill(),
                Rect {
                    x: 0.0,
                    y: label_box_y,
                    w: t_sz.x,
                    h: 80.0,
                },
                8.0,
                Color::BLACK,
            )?;
            canvas.draw(&label_box, Vec2::new(90.0, 0.0));

            // Currently selected
            if i == self.currently_selected || Some(i) == self.should_animate_from {
                // We know out_animating_selection will automatically be none if in_animating_selection is none.
                if self.in_animating_selection.is_none() && !self.animation_finished {
                    self.in_animating_selection = Some(keyframes![
                        (0.0, 0.0, EaseInOut),
                        (t_sz.x - 10.0, 0.5, EaseInOut)
                    ]);
                    self.out_animating_selection = Some(keyframes![
                        (t_sz.x - 10.0, 0.0, EaseInOut),
                        (0.0, 0.5, EaseInOut)
                    ]);
                }
                let mut delete_animating_selection = false;
                let selection_box = Mesh::new_rounded_rectangle(
                    ctx,
                    DrawMode::fill(),
                    Rect {
                        x: 0.0 + 5.0,
                        y: label_box_y + 5.0,
                        w: if !self.animation_finished {
                            let se = if Some(i) == self.should_animate_from {
                                if let Some(se) = &mut self.out_animating_selection {
                                    se
                                } else {
                                    // Should never happen
                                    panic!()
                                }
                            } else if let Some(se) = &mut self.in_animating_selection {
                                if se.finished() {
                                    delete_animating_selection = true;
                                }
                                se
                            } else {
                                // Should never happen
                                panic!()
                            };
                            se.advance_by(0.05);
                            se.now()
                        } else {
                            t_sz.x - 10.0
                        },
                        h: 70.0,
                    },
                    5.0,
                    Color::WHITE,
                )?;
                canvas.draw(&selection_box, Vec2::new(90.0, 0.0));
                // We have to use a separate variable for borrow checking reasons
                if delete_animating_selection {
                    self.in_animating_selection = None;
                    self.out_animating_selection = None;
                    self.should_animate_from = None;
                    self.animation_finished = true;
                }
            }

            // Draw text
            let menu_text = Text::new(TextFragment {
                text: menu_mapping.text.clone(),
                color: Some(if i == self.currently_selected {
                    Color::BLACK
                } else {
                    Color::WHITE
                }),
                font: Some("SecularOne-Regular".into()),
                scale: Some(PxScale::from(48.0)),
            });

            let mt_sz = menu_text
                .measure(ctx)
                .expect("Failed to calculate menu text size");

            let mt_y = label_box_y + ((80.0 - mt_sz.y) / 2.0);
            canvas.draw(&menu_text, graphics::DrawParam::from([110.0, mt_y]));
        }

        Ok(())
    }
}

impl Scene for GameMenu {
    fn handle_key_event(&mut self, _ctx: &mut Context, key_input: KeyInput, _repeat: bool) {
        if self.in_animating_selection.is_none() {
            if let Some(vkeycode) = key_input.keycode {
                match vkeycode {
                    VirtualKeyCode::Up => {
                        if self.currently_selected > 0 {
                            self.should_animate_from = Some(self.currently_selected);
                            self.animation_finished = false;
                            self.currently_selected -= 1;
                        }
                    }

                    VirtualKeyCode::Down => {
                        if self.currently_selected < self.menu_mappings.len() - 1 {
                            self.should_animate_from = Some(self.currently_selected);
                            self.animation_finished = false;
                            self.currently_selected += 1
                        }
                    }
                    VirtualKeyCode::Return => {
                        self.to_next_scene = true;
                    }
                    _ => {}
                }
            }
        }
    }

    fn next_scene(&self, ctx: &mut Context) -> Option<Box<dyn Scene>> {
        if self.to_next_scene {
            Some((self.menu_mappings[self.currently_selected].next_page)(ctx))
        } else {
            None
        }
    }
}
