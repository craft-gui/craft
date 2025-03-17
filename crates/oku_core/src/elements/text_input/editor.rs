use core::default::Default;
use parley::{GenericFamily, StyleProperty};
use std::time::{Duration, Instant};
use vello::peniko::Brush;
use winit::{
    event::{Ime, Modifiers},
    keyboard::{Key, NamedKey},
};

use crate::elements::text_input::driver::PlainEditorDriver;
use crate::elements::text_input::plain_text_editor::PlainEditor;
use crate::events::OkuMessage;
use crate::style::Style;
use parley::{FontContext, LayoutContext};

pub struct Editor {
    pub(crate) font_cx: FontContext,
    pub(crate) layout_cx: LayoutContext<Brush>,
    pub(crate) editor: PlainEditor<Brush>,
    pub(crate) last_click_time: Option<Instant>,
    pub(crate) click_count: u32,
    pub(crate) pointer_down: bool,
    pub(crate) cursor_pos: (f32, f32),
    pub(crate) cursor_visible: bool,
    pub(crate) modifiers: Option<Modifiers>,
    pub(crate) start_time: Option<Instant>,
    pub(crate) blink_period: Duration,
}

impl Editor {
    pub fn new(text: &str, style: Style) -> Self {
        let mut editor = PlainEditor::new(32.0);
        editor.set_text(text);
        editor.set_scale(1.0);

        let text_brush = Brush::Solid(style.color());

        //// Append the element's font family.
        //if let Some(font_family) = style.font_family() {
        //    if let Some(font_family) = FontFamily::parse(font_family) {
        //        font_families.push(font_family);
        //    }
        //};

        // let family_names = get_fallback_font_families(font_context);
        // // Append the fallback fonts.
        // {
        //     for family_name in &family_names {
        //         font_families.push(FontFamily::parse(family_name).unwrap());
        //     }
        // }

        //let font_stack = FontStack::from(font_families.as_slice());

        let styles = editor.edit_styles();
        styles.insert(StyleProperty::LineHeight(1.5));
        //styles.insert(StyleProperty::FontStack(font_stack));
        styles.insert(StyleProperty::FontSize(style.font_size()));
        styles.insert(StyleProperty::FontWeight(parley::FontWeight::new(style.font_weight().0 as f32)));
        styles.insert(GenericFamily::SystemUi.into());
        styles.insert(StyleProperty::Brush(text_brush));

        Self {
            font_cx: Default::default(),
            layout_cx: Default::default(),
            editor,
            last_click_time: Default::default(),
            click_count: Default::default(),
            pointer_down: Default::default(),
            cursor_pos: Default::default(),
            cursor_visible: true,
            modifiers: Default::default(),
            start_time: Default::default(),
            blink_period: Default::default(),
        }
    }

    fn driver(&mut self) -> PlainEditorDriver<'_, Brush> {
        self.editor.driver(&mut self.font_cx, &mut self.layout_cx)
    }

    pub fn cursor_reset(&mut self) {
        self.start_time = Some(Instant::now());
        // TODO: for real world use, this should be reading from the system settings
        self.blink_period = Duration::from_millis(500);
        self.cursor_visible = true;
    }

    pub fn handle_event(&mut self, event: OkuMessage, text_x: f32, text_y: f32) {
        match event {
            // WindowEvent::ModifiersChanged(modifiers) => {
            //     self.modifiers = Some(modifiers);
            // }
            OkuMessage::KeyboardInputEvent(keyboard_input) if !self.editor.is_composing() => {
                if !keyboard_input.event.state.is_pressed() {
                    return;
                }
                self.cursor_reset();
                let mut drv = self.editor.driver(&mut self.font_cx, &mut self.layout_cx);
                #[allow(unused)]
                let (shift, action_mod) = self
                    .modifiers
                    .map(|mods| {
                        (
                            mods.state().shift_key(),
                            if cfg!(target_os = "macos") {
                                mods.state().super_key()
                            } else {
                                mods.state().control_key()
                            },
                        )
                    })
                    .unwrap_or_default();

                match keyboard_input.event.logical_key {
                    // #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
                    // Key::Character(c) if action_mod && matches!(c.as_str(), "c" | "x" | "v") => {
                    //     use clipboard_rs::{Clipboard, ClipboardContext};
                    //     match c.to_lowercase().as_str() {
                    //         "c" => {
                    //             if let Some(text) = drv.editor.selected_text() {
                    //                 let cb = ClipboardContext::new().unwrap();
                    //                 cb.set_text(text.to_owned()).ok();
                    //             }
                    //         }
                    //         "x" => {
                    //             if let Some(text) = drv.editor.selected_text() {
                    //                 let cb = ClipboardContext::new().unwrap();
                    //                 cb.set_text(text.to_owned()).ok();
                    //                 drv.delete_selection();
                    //             }
                    //         }
                    //         "v" => {
                    //             let cb = ClipboardContext::new().unwrap();
                    //             let text = cb.get_text().unwrap_or_default();
                    //             drv.insert_or_replace_selection(&text);
                    //         }
                    //         _ => (),
                    //     }
                    // }
                    Key::Character(c) if action_mod && matches!(c.to_lowercase().as_str(), "a") => {
                        if shift {
                            drv.collapse_selection();
                        } else {
                            drv.select_all();
                        }
                    }
                    Key::Named(NamedKey::ArrowLeft) => {
                        if action_mod {
                            if shift {
                                drv.select_word_left();
                            } else {
                                drv.move_word_left();
                            }
                        } else if shift {
                            drv.select_left();
                        } else {
                            drv.move_left();
                        }
                    }
                    Key::Named(NamedKey::ArrowRight) => {
                        if action_mod {
                            if shift {
                                drv.select_word_right();
                            } else {
                                drv.move_word_right();
                            }
                        } else if shift {
                            drv.select_right();
                        } else {
                            drv.move_right();
                        }
                    }
                    Key::Named(NamedKey::ArrowUp) => {
                        if shift {
                            drv.select_up();
                        } else {
                            drv.move_up();
                        }
                    }
                    Key::Named(NamedKey::ArrowDown) => {
                        if shift {
                            drv.select_down();
                        } else {
                            drv.move_down();
                        }
                    }
                    Key::Named(NamedKey::Home) => {
                        if action_mod {
                            if shift {
                                drv.select_to_text_start();
                            } else {
                                drv.move_to_text_start();
                            }
                        } else if shift {
                            drv.select_to_line_start();
                        } else {
                            drv.move_to_line_start();
                        }
                    }
                    Key::Named(NamedKey::End) => {
                        let this = &mut *self;
                        let mut drv = this.driver();

                        if action_mod {
                            if shift {
                                drv.select_to_text_end();
                            } else {
                                drv.move_to_text_end();
                            }
                        } else if shift {
                            drv.select_to_line_end();
                        } else {
                            drv.move_to_line_end();
                        }
                    }
                    Key::Named(NamedKey::Delete) => {
                        if action_mod {
                            drv.delete_word();
                        } else {
                            drv.delete();
                        }
                    }
                    Key::Named(NamedKey::Backspace) => {
                        if action_mod {
                            drv.backdelete_word();
                        } else {
                            drv.backdelete();
                        }
                    }
                    Key::Named(NamedKey::Enter) => {
                        drv.insert_or_replace_selection("\n");
                    }
                    Key::Named(NamedKey::Space) => {
                        drv.insert_or_replace_selection(" ");
                    }
                    Key::Character(s) => {
                        drv.insert_or_replace_selection(&s);
                    }
                    _ => (),
                }
            }
            // WindowEvent::Touch(Touch {
            //     phase, location, ..
            // }) if !self.editor.is_composing() => {
            //     let mut drv = self.editor.driver(&mut self.font_cx, &mut self.layout_cx);
            //     use winit::event::TouchPhase::*;
            //     match phase {
            //         Started => {
            //             // TODO: start a timer to convert to a SelectWordAtPoint
            //             drv.move_to_point(location.x as f32, location.y as f32);
            //         }
            //         Cancelled => {
            //             drv.collapse_selection();
            //         }
            //         Moved => {
            //             // TODO: cancel SelectWordAtPoint timer
            //             drv.extend_selection_to_point(
            //                 location.x as f32,
            //                 location.y as f32,
            //             );
            //         }
            //         Ended => (),
            //     }
            // }
            OkuMessage::PointerButtonEvent(pointer_button) => {
                if pointer_button.button.mouse_button() == winit::event::MouseButton::Left {
                    self.pointer_down = pointer_button.state.is_pressed();
                    self.cursor_reset();
                    if self.pointer_down && !self.editor.is_composing() {
                        let now = Instant::now();
                        if let Some(last) = self.last_click_time.take() {
                            if now.duration_since(last).as_secs_f64() < 0.25 {
                                self.click_count = (self.click_count + 1) % 4;
                            } else {
                                self.click_count = 1;
                            }
                        } else {
                            self.click_count = 1;
                        }
                        self.last_click_time = Some(now);
                        let click_count = self.click_count;
                        let cursor_pos = self.cursor_pos;
                        let mut drv = self.editor.driver(&mut self.font_cx, &mut self.layout_cx);
                        match click_count {
                            2 => drv.select_word_at_point(cursor_pos.0, cursor_pos.1),
                            3 => drv.select_line_at_point(cursor_pos.0, cursor_pos.1),
                            _ => drv.move_to_point(cursor_pos.0, cursor_pos.1),
                        }
                    }
                }
            }
            OkuMessage::PointerMovedEvent(pointer_moved) => {
                let prev_pos = self.cursor_pos;
                // NOTE: Cursor position should be relative to the top left of the text box.
                self.cursor_pos = (pointer_moved.position.x - text_x, pointer_moved.position.y - text_y);
                // macOS seems to generate a spurious move after selecting word?
                if self.pointer_down && prev_pos != self.cursor_pos && !self.editor.is_composing() {
                    self.cursor_reset();
                    let cursor_pos = self.cursor_pos;
                    self.driver().extend_selection_to_point(cursor_pos.0, cursor_pos.1);
                }
            }
            OkuMessage::ImeEvent(Ime::Disabled) => {
                self.driver().clear_compose();
            }
            OkuMessage::ImeEvent(Ime::Commit(text)) => {
                self.driver().insert_or_replace_selection(&text);
            }
            OkuMessage::ImeEvent(Ime::Preedit(text, cursor)) => {
                if text.is_empty() {
                    self.driver().clear_compose();
                } else {
                    self.driver().set_compose(&text, cursor);
                }
            }
            _ => {}
        }
    }

    //pub fn handle_accesskit_action_request(&mut self, req: &accesskit::ActionRequest) {
    //    if req.action == accesskit::Action::SetTextSelection {
    //        if let Some(accesskit::ActionData::SetTextSelection(selection)) = &req.data {
    //            self.driver().select_from_accesskit(selection);
    //        }
    //    }
    //}

    //pub fn accessibility(&mut self, update: &mut TreeUpdate, node: &mut Node) {
    //    let mut drv = self.editor.driver(&mut self.font_cx, &mut self.layout_cx);
    //    drv.accessibility(update, node, next_node_id, 0.0, 0.0);
    //}
}
