use craft_retained::elements::{AsElement, DynElement, ScrollOptions, ScrollState};
use craft_retained::events::ui_events::pointer::PointerId;
use craft_retained::events::{KeyboardInputHandler, PointerCaptureHandler, PointerEnterHandler, PointerEventHandler, PointerLeaveHandler, PointerUpdateHandler, ScrollHandler, SliderValueChangedHandler};
use craft_retained::geometry::ElementBox;
use craft_retained::style::{AlignItems, BoxShadow, BoxSizing, Display, FlexDirection, FlexWrap, FontFamily, FontStyle, FontWeight, JustifyContent, Overflow, Position, ScrollbarColor, TextAlign, Underline, Unit};
use craft_retained::winit::dpi::PhysicalPosition;
use craft_retained::winit::event::WindowEvent::{CursorMoved, MouseInput};
use craft_retained::winit::event::{DeviceId, ElementState, MouseButton};
use craft_retained::{Color, CraftError, queue_window_event};

use crate::signals::Bindable;

/// Exposes a fluent/builder-pattern like API for elements.
/// Setters in this trait return Self and have no prefix.
/// Getters in this trait return specific data and have a get prefix.
pub trait Element: Clone + AsElement {
    fn get_children(&self) -> Vec<DynElement> {
        self.as_element_rc()
            .borrow()
            .children()
            .iter()
            .cloned()
            .map(DynElement::new)
            .collect()
    }

    fn get_previous_sibling(&self) -> Result<DynElement, CraftError> {
        self.as_element_rc()
            .borrow()
            .get_previous_sibling()
            .map(DynElement::new)
    }

    fn get_next_sibling(&self) -> Result<DynElement, CraftError> {
        self.as_element_rc().borrow().get_next_sibling().map(DynElement::new)
    }

    fn get_parent(&self) -> Result<DynElement, CraftError> {
        let parent = self.as_element_rc().borrow().parent();
        if let Some(parent) = parent {
            parent.upgrade().ok_or(CraftError::ElementNotFound).map(DynElement::new)
        } else {
            Err(CraftError::ElementNotFound)
        }
    }

    fn get_first_child(&self) -> Result<DynElement, CraftError> {
        self.as_element_rc().borrow().get_first_child().map(DynElement::new)
    }

    fn get_last_child(&self) -> Result<DynElement, CraftError> {
        self.as_element_rc().borrow().get_last_child().map(DynElement::new)
    }

    fn remove_child(&self, child: DynElement) -> Result<DynElement, CraftError> {
        self.as_element_rc()
            .borrow_mut()
            .remove_child(child.inner)
            .map(DynElement::new)
    }

    fn remove_all_children(&self) {
        self.as_element_rc().borrow_mut().remove_all_children()
    }

    fn swap_child(&self, child_1: DynElement, child_2: DynElement) -> Result<(), CraftError> {
        self.as_element_rc()
            .borrow_mut()
            .swap_child(child_1.inner, child_2.inner)
    }

    fn push(self, child: impl AsElement) -> Self {
        let child_rc = child.as_element_rc();
        self.as_element_rc().borrow_mut().push(child_rc);
        self
    }

    fn on_pointer_enter(self, on_pointer_enter: PointerEnterHandler) -> Self {
        self.as_element_rc().borrow_mut().on_pointer_enter(on_pointer_enter);
        self
    }

    fn on_pointer_leave(self, on_pointer_leave: PointerLeaveHandler) -> Self {
        self.as_element_rc().borrow_mut().on_pointer_leave(on_pointer_leave);
        self
    }

    fn id(self, id: &str) -> Self {
        self.as_element_rc().borrow_mut().set_id(id);
        self
    }

    fn get_id(&self) -> Option<String> {
        self.as_element_rc().borrow().get_id().map(|s| s.to_string())
    }

    fn on_pointer_button_down(self, on_pointer_button_down: PointerEventHandler) -> Self {
        self.as_element_rc()
            .borrow_mut()
            .on_pointer_button_down(on_pointer_button_down);
        self
    }

    fn on_pointer_moved(self, on_pointer_moved: PointerUpdateHandler) -> Self {
        self.as_element_rc().borrow_mut().on_pointer_moved(on_pointer_moved);
        self
    }

    fn on_pointer_button_up(self, on_pointer_button_up: PointerEventHandler) -> Self {
        self.as_element_rc()
            .borrow_mut()
            .on_pointer_button_up(on_pointer_button_up);
        self
    }

    fn on_lost_pointer_capture(self, on_lost_pointer_capture: PointerCaptureHandler) -> Self {
        self.as_element_rc()
            .borrow_mut()
            .on_lost_pointer_capture(on_lost_pointer_capture);
        self
    }

    fn on_got_pointer_capture(self, on_got_pointer_capture: PointerCaptureHandler) -> Self {
        self.as_element_rc()
            .borrow_mut()
            .on_got_pointer_capture(on_got_pointer_capture);
        self
    }

    fn on_keyboard_input(self, on_keyboard_input: KeyboardInputHandler) -> Self {
        self.as_element_rc().borrow_mut().on_keyboard_input(on_keyboard_input);
        self
    }

    fn on_slider_value_changed(self, on_slider_value_changed: SliderValueChangedHandler) -> Self {
        self.as_element_rc()
            .borrow_mut()
            .on_slider_value_changed(on_slider_value_changed);
        self
    }

    fn on_scroll(self, on_scroll: ScrollHandler) -> Self {
        self.as_element_rc().borrow_mut().on_scroll(on_scroll);
        self
    }

    fn scroll_to_child_by_id(self, id: &str) -> Self {
        self.as_element_rc()
            .borrow_mut()
            .scroll_to_child_by_id_with_options(id, ScrollOptions::default());
        self
    }

    fn scroll_to_child_by_id_with_options(self, id: &str, options: ScrollOptions) -> Self {
        self.as_element_rc()
            .borrow_mut()
            .scroll_to_child_by_id_with_options(id, options);
        self
    }

    fn scroll_to(self, y: f32) -> Self {
        self.as_element_rc().borrow_mut().scroll_to(y);
        self
    }

    fn scroll_to_top(self) -> Self {
        self.as_element_rc().borrow_mut().scroll_to_top();
        self
    }

    fn scroll_to_bottom(self) -> Self {
        self.as_element_rc().borrow_mut().scroll_to_bottom();
        self
    }

    fn scroll_by(self, y: f32) -> Self {
        self.as_element_rc().borrow_mut().scroll_by(y);
        self
    }

    fn get_scroll_state(&self) -> ScrollState {
        self.as_element_rc().borrow().get_scroll_state()
    }

    fn display(self, display: impl Bindable<Display>) -> Self {
        let element = self.as_element_rc();
        display.bind(move |value| {
            element.borrow_mut().set_display(value);
        });
        self
    }

    fn box_sizing(self, box_sizing: impl Bindable<BoxSizing>) -> Self {
        let element = self.as_element_rc();
        box_sizing.bind(move |value| {
            element.borrow_mut().set_box_sizing(value);
        });
        self
    }

    fn position(self, position: impl Bindable<Position>) -> Self {
        let element = self.as_element_rc();
        position.bind(move |value| {
            element.borrow_mut().set_position(value);
        });
        self
    }

    fn margin(
        self,
        top: impl Bindable<Unit> + Clone,
        right: impl Bindable<Unit> + Clone,
        bottom: impl Bindable<Unit> + Clone,
        left: impl Bindable<Unit> + Clone,
    ) -> Self {
        let element = self.as_element_rc();

        let r0 = right.clone();
        let b0 = bottom.clone();
        let l0 = left.clone();

        top.bind(move |t| {
            let element = element.clone();
            let r = r0.clone();
            let b = b0.clone();
            let l = l0.clone();

            r.bind(move |r| {
                let element = element.clone();
                let b = b.clone();
                let l = l.clone();

                b.bind(move |b| {
                    let element = element.clone();
                    let l = l.clone();

                    l.bind(move |l| {
                        element.borrow_mut().set_margin(t, r, b, l);
                    });
                });
            });
        });

        self
    }

    fn margin_all(self, value: impl Bindable<Unit> + Clone) -> Self {
        let element = self.as_element_rc();

        value.bind(move |v| {
            element.borrow_mut().set_margin_all(v);
        });

        self
    }

    fn margin_vertical(self, value: impl Bindable<Unit> + Clone) -> Self {
        let element = self.as_element_rc();

        value.bind(move |v| {
            element.borrow_mut().set_margin_vertical(v);
        });

        self
    }

    fn margin_horizontal(self, value: impl Bindable<Unit> + Clone) -> Self {
        let element = self.as_element_rc();

        value.bind(move |v| {
            element.borrow_mut().set_margin_horizontal(v);
        });

        self
    }

    fn padding(
        self,
        top: impl Bindable<Unit> + Clone,
        right: impl Bindable<Unit> + Clone,
        bottom: impl Bindable<Unit> + Clone,
        left: impl Bindable<Unit> + Clone,
    ) -> Self {
        let element = self.as_element_rc();

        let r0 = right.clone();
        let b0 = bottom.clone();
        let l0 = left.clone();

        top.bind(move |t| {
            let element = element.clone();
            let r = r0.clone();
            let b = b0.clone();
            let l = l0.clone();

            r.bind(move |r| {
                let element = element.clone();
                let b = b.clone();
                let l = l.clone();

                b.bind(move |b| {
                    let element = element.clone();
                    let l = l.clone();

                    l.bind(move |l| {
                        element.borrow_mut().set_padding(t, r, b, l);
                    });
                });
            });
        });

        self
    }

    fn padding_all(self, value: impl Bindable<Unit> + Clone) -> Self {
        let element = self.as_element_rc();

        value.bind(move |v| {
            element.borrow_mut().set_padding_all(v);
        });

        self
    }

    fn padding_vertical(self, value: impl Bindable<Unit> + Clone) -> Self {
        let element = self.as_element_rc();

        value.bind(move |v| {
            element.borrow_mut().set_padding_vertical(v);
        });

        self
    }

    fn padding_horizontal(self, value: impl Bindable<Unit> + Clone) -> Self {
        let element = self.as_element_rc();

        value.bind(move |v| {
            element.borrow_mut().set_padding_horizontal(v);
        });

        self
    }

    fn gap(self, column_gap: impl Bindable<Unit> + Clone, row_gap: impl Bindable<Unit> + Clone) -> Self {
        let element = self.as_element_rc();

        let c0 = column_gap.clone();

        row_gap.bind(move |r| {
            let element = element.clone();
            let c = c0.clone();

            c.bind(move |c| {
                element.borrow_mut().set_gap(c, r);
            });
        });

        self
    }

    fn row_gap(self, value: impl Bindable<Unit> + Clone) -> Self {
        let element = self.as_element_rc();

        value.bind(move |v| {
            element.borrow_mut().set_row_gap(v);
        });

        self
    }

    fn column_gap(self, value: impl Bindable<Unit> + Clone) -> Self {
        let element = self.as_element_rc();

        value.bind(move |v| {
            element.borrow_mut().set_column_gap(v);
        });

        self
    }

    fn inset(
        self,
        top: impl Bindable<Unit> + Clone,
        right: impl Bindable<Unit> + Clone,
        bottom: impl Bindable<Unit> + Clone,
        left: impl Bindable<Unit> + Clone,
    ) -> Self {
        let element = self.as_element_rc();

        let r0 = right.clone();
        let b0 = bottom.clone();
        let l0 = left.clone();

        top.bind(move |t| {
            let element = element.clone();
            let r = r0.clone();
            let b = b0.clone();
            let l = l0.clone();

            r.bind(move |r| {
                let element = element.clone();
                let b = b.clone();
                let l = l.clone();

                b.bind(move |b| {
                    let element = element.clone();
                    let l = l.clone();

                    l.bind(move |l| {
                        element.borrow_mut().set_inset(t, r, b, l);
                    });
                });
            });
        });

        self
    }

    fn min_width(self, min_width: impl Bindable<Unit>) -> Self {
        let element = self.as_element_rc();
        min_width.bind(move |value| {
            element.borrow_mut().set_min_width(value);
        });
        self
    }

    fn min_height(self, min_height: impl Bindable<Unit>) -> Self {
        let element = self.as_element_rc();
        min_height.bind(move |value| {
            element.borrow_mut().set_min_height(value);
        });
        self
    }

    fn width(self, width: impl Bindable<Unit>) -> Self {
        let element = self.as_element_rc();
        width.bind(move |value| {
            element.borrow_mut().set_width(value);
        });
        self
    }

    fn height(self, height: impl Bindable<Unit>) -> Self {
        let element = self.as_element_rc();
        height.bind(move |value| {
            element.borrow_mut().set_height(value);
        });
        self
    }

    fn max_width(self, max_width: impl Bindable<Unit>) -> Self {
        let element = self.as_element_rc();
        max_width.bind(move |value| {
            element.borrow_mut().set_max_width(value);
        });
        self
    }

    fn max_height(self, max_height: impl Bindable<Unit>) -> Self {
        let element = self.as_element_rc();
        max_height.bind(move |value| {
            element.borrow_mut().set_max_height(value);
        });
        self
    }

    fn wrap(self, wrap: impl Bindable<FlexWrap>) -> Self {
        let element = self.as_element_rc();
        wrap.bind(move |value| {
            element.borrow_mut().set_wrap(value);
        });
        self
    }

    fn align_items(self, align_items: impl Bindable<Option<AlignItems>>) -> Self {
        let element = self.as_element_rc();
        align_items.bind(move |value| {
            element.borrow_mut().set_align_items(value);
        });
        self
    }

    fn justify_content(self, justify_content: impl Bindable<Option<JustifyContent>>) -> Self {
        let element = self.as_element_rc();
        justify_content.bind(move |value| {
            element.borrow_mut().set_justify_content(value);
        });
        self
    }

    fn flex_direction(self, flex_direction: impl Bindable<FlexDirection>) -> Self {
        let element = self.as_element_rc();
        flex_direction.bind(move |value| {
            element.borrow_mut().set_flex_direction(value);
        });
        self
    }

    fn flex_grow(self, flex_grow: impl Bindable<f32>) -> Self {
        let element = self.as_element_rc();
        flex_grow.bind(move |value| {
            element.borrow_mut().set_flex_grow(value);
        });
        self
    }

    fn flex_shrink(self, flex_shrink: impl Bindable<f32>) -> Self {
        let element = self.as_element_rc();
        flex_shrink.bind(move |value| {
            element.borrow_mut().set_flex_shrink(value);
        });
        self
    }

    fn flex_basis(self, flex_basis: impl Bindable<Unit>) -> Self {
        let element = self.as_element_rc();
        flex_basis.bind(move |value| {
            element.borrow_mut().set_flex_basis(value);
        });
        self
    }

    fn font_family(self, font_family: impl Bindable<FontFamily>) -> Self {
        let element = self.as_element_rc();
        font_family.bind(move |value| {
            element.borrow_mut().set_font_family(value);
        });
        self
    }

    fn color(self, color: impl Bindable<Color>) -> Self {
        let element = self.as_element_rc();
        color.bind(move |v| element.borrow_mut().set_color(v));
        self
    }

    fn background_color(self, background_color: impl Bindable<Color>) -> Self {
        let element = self.as_element_rc();
        background_color.bind(move |value| {
            element.borrow_mut().set_background_color(value);
        });
        self
    }

    fn font_size(self, font_size: impl Bindable<f32>) -> Self {
        let element = self.as_element_rc();
        font_size.bind(move |v| element.borrow_mut().set_font_size(v));
        self
    }

    fn line_height(self, line_height: impl Bindable<f32>) -> Self {
        let element = self.as_element_rc();
        line_height.bind(move |v| element.borrow_mut().set_line_height(v));
        self
    }

    fn font_weight(self, font_weight: impl Bindable<FontWeight>) -> Self {
        let element = self.as_element_rc();
        font_weight.bind(move |v| element.borrow_mut().set_font_weight(v));
        self
    }

    fn font_style(self, font_style: impl Bindable<FontStyle>) -> Self {
        let element = self.as_element_rc();
        font_style.bind(move |v| element.borrow_mut().set_font_style(v));
        self
    }

    fn text_align(self, text_align: impl Bindable<TextAlign>) -> Self {
        let element = self.as_element_rc();
        text_align.bind(move |v| element.borrow_mut().set_text_align(v));
        self
    }

    fn underline(self, underline: impl Bindable<Option<Underline>>) -> Self {
        let element = self.as_element_rc();
        underline.bind(move |v| element.borrow_mut().set_underline(v));
        self
    }

    fn overflow(
        self,
        overflow_x: impl Bindable<Overflow> + Clone,
        overflow_y: impl Bindable<Overflow> + Clone,
    ) -> Self {
        let element = self.as_element_rc();

        overflow_x.bind(move |x| {
            let element = element.clone();
            let overflow_y = overflow_y.clone();
            overflow_y.bind(move |y| {
                element.borrow_mut().set_overflow(x, y);
            });
        });

        self
    }

    fn overflow_x(self, value: impl Bindable<Overflow> + Clone) -> Self {
        let element = self.as_element_rc();
        value.bind(move |x| {
            element.borrow_mut().set_overflow_x(x);
        });
        self
    }

    fn overflow_y(self, value: impl Bindable<Overflow> + Clone) -> Self {
        let element = self.as_element_rc();
        value.bind(move |y| {
            element.borrow_mut().set_overflow_y(y);
        });
        self
    }

    fn border_color(
        self,
        top: impl Bindable<Color> + Clone,
        right: impl Bindable<Color> + Clone,
        bottom: impl Bindable<Color> + Clone,
        left: impl Bindable<Color> + Clone,
    ) -> Self {
        let element = self.as_element_rc();

        let r0 = right.clone();
        let b0 = bottom.clone();
        let l0 = left.clone();

        top.bind(move |t| {
            let element = element.clone();
            let r = r0.clone();
            let b = b0.clone();
            let l = l0.clone();

            r.bind(move |r| {
                let element = element.clone();
                let b = b.clone();
                let l = l.clone();

                b.bind(move |b| {
                    let element = element.clone();
                    let l = l.clone();

                    l.bind(move |l| {
                        element.borrow_mut().set_border_color(t, r, b, l);
                    });
                });
            });
        });

        self
    }

    fn border_color_all(self, color: impl Bindable<Color> + Clone) -> Self {
        let element = self.as_element_rc();
        color.bind(move |c| {
            element.borrow_mut().set_border_color_all(c);
        });
        self
    }

    fn border_color_vertical(self, color: impl Bindable<Color> + Clone) -> Self {
        let element = self.as_element_rc();
        color.bind(move |c| {
            element.borrow_mut().set_border_color_vertical(c);
        });
        self
    }

    fn border_color_horizontal(self, color: impl Bindable<Color> + Clone) -> Self {
        let element = self.as_element_rc();
        color.bind(move |c| {
            element.borrow_mut().set_border_color_horizontal(c);
        });
        self
    }

    fn border_width(
        self,
        top: impl Bindable<Unit> + Clone,
        right: impl Bindable<Unit> + Clone,
        bottom: impl Bindable<Unit> + Clone,
        left: impl Bindable<Unit> + Clone,
    ) -> Self {
        let element = self.as_element_rc();

        let r0 = right.clone();
        let b0 = bottom.clone();
        let l0 = left.clone();

        top.bind(move |t| {
            let element = element.clone();
            let r = r0.clone();
            let b = b0.clone();
            let l = l0.clone();

            r.bind(move |r| {
                let element = element.clone();
                let b = b.clone();
                let l = l.clone();

                b.bind(move |b| {
                    let element = element.clone();
                    let l = l.clone();

                    l.bind(move |l| {
                        element.borrow_mut().set_border_width(t, r, b, l);
                    });
                });
            });
        });

        self
    }

    fn border_width_all(self, value: impl Bindable<Unit> + Clone) -> Self {
        let element = self.as_element_rc();
        value.bind(move |v| {
            element.borrow_mut().set_border_width_all(v);
        });
        self
    }

    fn border_width_vertical(self, value: impl Bindable<Unit> + Clone) -> Self {
        let element = self.as_element_rc();
        value.bind(move |v| {
            element.borrow_mut().set_border_width_vertical(v);
        });
        self
    }

    fn border_width_horizontal(self, value: impl Bindable<Unit> + Clone) -> Self {
        let element = self.as_element_rc();
        value.bind(move |v| {
            element.borrow_mut().set_border_width_horizontal(v);
        });
        self
    }


    fn border_radius(
        self,
        top: impl Bindable<(f32, f32)> + Clone,
        right: impl Bindable<(f32, f32)> + Clone,
        bottom: impl Bindable<(f32, f32)> + Clone,
        left: impl Bindable<(f32, f32)> + Clone,
    ) -> Self {
        let element = self.as_element_rc();

        let r0 = right.clone();
        let b0 = bottom.clone();
        let l0 = left.clone();

        top.bind(move |t| {
            let element = element.clone();
            let r = r0.clone();
            let b = b0.clone();
            let l = l0.clone();

            r.bind(move |r| {
                let element = element.clone();
                let b = b.clone();
                let l = l.clone();

                b.bind(move |b| {
                    let element = element.clone();
                    let l = l.clone();

                    l.bind(move |l| {
                        element.borrow_mut().set_border_radius(t, r, b, l);
                    });
                });
            });
        });

        self
    }

    fn border_radius_all(self, value: impl Bindable<(f32, f32)> + Clone) -> Self {
        let element = self.as_element_rc();
        value.bind(move |v| {
            element.borrow_mut().set_border_radius_all(v);
        });
        self
    }

    fn border_radius_vertical(self, value: impl Bindable<(f32, f32)> + Clone) -> Self {
        let element = self.as_element_rc();
        value.bind(move |v| {
            element.borrow_mut().set_border_radius_vertical(v);
        });
        self
    }

    fn border_radius_horizontal(self, value: impl Bindable<(f32, f32)> + Clone) -> Self {
        let element = self.as_element_rc();
        value.bind(move |v| {
            element.borrow_mut().set_border_radius_horizontal(v);
        });
        self
    }

    fn scrollbar_color(self, scrollbar_color: impl Bindable<ScrollbarColor>) -> Self {
        let element = self.as_element_rc();
        scrollbar_color.bind(move |v| {
            element.borrow_mut().set_scrollbar_color(v);
        });
        self
    }

    fn scrollbar_thumb_margin(
        self,
        top: impl Bindable<f32> + Clone,
        right: impl Bindable<f32> + Clone,
        bottom: impl Bindable<f32> + Clone,
        left: impl Bindable<f32> + Clone,
    ) -> Self {
        let element = self.as_element_rc();

        let r0 = right.clone();
        let b0 = bottom.clone();
        let l0 = left.clone();

        top.bind(move |t| {
            let element = element.clone();
            let r = r0.clone();
            let b = b0.clone();
            let l = l0.clone();

            r.bind(move |r| {
                let element = element.clone();
                let b = b.clone();
                let l = l.clone();

                b.bind(move |b| {
                    let element = element.clone();
                    let l = l.clone();

                    l.bind(move |l| {
                        element.borrow_mut().set_scrollbar_thumb_margin(t, r, b, l);
                    });
                });
            });
        });

        self
    }

    fn set_scrollbar_thumb_radius(
        self,
        top: impl Bindable<(f32, f32)> + Clone,
        right: impl Bindable<(f32, f32)> + Clone,
        bottom: impl Bindable<(f32, f32)> + Clone,
        left: impl Bindable<(f32, f32)> + Clone,
    ) -> Self {
        let element = self.as_element_rc();

        let r0 = right.clone();
        let b0 = bottom.clone();
        let l0 = left.clone();

        top.bind(move |t| {
            let element = element.clone();
            let r = r0.clone();
            let b = b0.clone();
            let l = l0.clone();

            r.bind(move |r| {
                let element = element.clone();
                let b = b.clone();
                let l = l.clone();

                b.bind(move |b| {
                    let element = element.clone();
                    let l = l.clone();

                    l.bind(move |l| {
                        element.borrow_mut().set_scrollbar_thumb_radius(t, r, b, l);
                    });
                });
            });
        });

        self
    }

    fn scrollbar_width(self, selection_color: impl Bindable<Color>) -> Self {
        let element = self.as_element_rc();
        selection_color.bind(move |v| {
            element.borrow_mut().set_selection_color(v);
        });
        self
    }

    fn box_shadows(self, box_shadows: impl Bindable<Vec<BoxShadow>>) -> Self {
        let element = self.as_element_rc();
        box_shadows.bind(move |v| {
            element.borrow_mut().set_box_shadows(v);
        });
        self
    }

    fn focus(self) -> Self {
        self.as_element_rc().borrow_mut().focus();
        self
    }

    fn is_focused(&self) -> bool {
        self.as_element_rc().borrow().is_focused()
    }

    fn unfocus(self) -> Self {
        self.as_element_rc().borrow_mut().unfocus();
        self
    }

    fn get_computed_box_transformed(&self) -> ElementBox {
        self.as_element_rc().borrow().get_computed_box_transformed()
    }

    fn has_pointer_capture(&self, pointer_id: PointerId) -> bool {
        self.as_element_rc().borrow().has_pointer_capture(pointer_id)
    }

    #[allow(async_fn_in_trait)]
    async fn click(&self) {
        let pos = self
            .as_element_rc()
            .borrow()
            .get_computed_box_transformed()
            .padding_rectangle();

        let mouse_move = CursorMoved {
            device_id: DeviceId::dummy(),
            position: PhysicalPosition::new(pos.x as f64, pos.y as f64),
        };

        let mouse_down = MouseInput {
            device_id: DeviceId::dummy(),
            state: ElementState::Pressed,
            button: MouseButton::Left,
        };

        let mouse_up = MouseInput {
            device_id: DeviceId::dummy(),
            state: ElementState::Released,
            button: MouseButton::Left,
        };

        let window_id = self.as_element_rc().borrow().get_winit_window().unwrap().id();

        queue_window_event(window_id, mouse_move);
        queue_window_event(window_id, mouse_down);
        queue_window_event(window_id, mouse_up);
    }
}
