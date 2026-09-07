use std::collections::VecDeque;
use std::sync::Arc;

use peniko::Color;

use retgui_primitives::brush::Brush;
use retgui_primitives::geometry::borders::CssRoundedRect;
use retgui_primitives::geometry::{Point, Rectangle, TrblRectangle, Vec2};
use retgui_primitives::gradient::Gradient;

use retgui_renderer::renderer::Renderer;

use retgui_resource_manager::ResourceManager;

use winit::event::ElementState;
use winit::keyboard::KeyCode;

use crate::elements::element_data::ElementData;
use crate::elements::traits::clone_element;
use crate::elements::{DynElement, Element, ElementIds, ElementInternals, ElementStates, HasElementData, RetGuiAccessTree, RetainedElements};
use crate::events::{Event, EventKind, SliderValueChangedEvent};
use crate::layout::GummyTree;
use crate::layout::layout::{CssComputedBorder, draw_borders_generic};
use crate::style::Unit;
use crate::text::text_context::TextContext;
use crate::{App, palette};

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub enum SliderDirection {
    #[default]
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy)]
pub struct Slider {
    pub(crate) inner: DynElement,
}

#[derive(Clone)]
pub(crate) struct SliderElement {
    element_data: ElementData,

    step: f64,
    min: f64,
    max: f64,
    direction: SliderDirection,
    value: f64,
    dragging: bool,

    // Thumb
    thumb_size: f64,
    thumb_background_brush: Brush,
    thumb_border_radius: Option<[(f32, f32); 4]>,

    // Track
    track_background_brush: Option<Brush>,
    track_border_radius: Option<[(f32, f32); 4]>,
}

impl Slider {
    pub fn new(app: &mut App, thumb_size: f32) -> Self {
        Self {
            inner: SliderElement::create(
                &mut app.elements,
                &mut app.gummy_tree,
                &app.access_tree,
                &mut app.by_internal_id,
                thumb_size,
            ),
        }
    }

    pub fn set_value(&self, app: &mut App, value: f64) {
        if let Some(slider) = app.try_get_as_mut::<SliderElement>(self.inner) {
            slider.set_value(value);
        }
    }

    pub fn value(&self, app: &App) -> f64 {
        app.try_get_as::<SliderElement>(self.inner)
            .map_or(0.0, SliderElement::get_value)
    }

    pub fn set_step(&self, app: &mut App, value: f64) {
        if let Some(slider) = app.try_get_as_mut::<SliderElement>(self.inner) {
            slider.set_step(value);
        }
    }

    pub fn step(&self, app: &App) -> f64 {
        app.try_get_as::<SliderElement>(self.inner)
            .map_or(0.0, SliderElement::get_step)
    }

    pub fn set_min(&self, app: &mut App, min: f64) {
        if let Some(slider) = app.try_get_as_mut::<SliderElement>(self.inner) {
            slider.set_min(min);
        }
    }

    pub fn min(&self, app: &App) -> f64 {
        app.try_get_as::<SliderElement>(self.inner)
            .map_or(0.0, SliderElement::get_min)
    }

    pub fn set_max(&self, app: &mut App, max: f64) {
        if let Some(slider) = app.try_get_as_mut::<SliderElement>(self.inner) {
            slider.set_max(max);
        }
    }

    pub fn max(&self, app: &App) -> f64 {
        app.try_get_as::<SliderElement>(self.inner)
            .map_or(0.0, SliderElement::get_max)
    }

    pub fn set_direction(&self, app: &mut App, direction: SliderDirection) {
        if let Some(slider) = app.try_get_as_mut::<SliderElement>(self.inner) {
            slider.set_direction(direction);
        }
    }

    pub fn direction(&self, app: &App) -> SliderDirection {
        app.try_get_as::<SliderElement>(self.inner)
            .map_or_else(SliderDirection::default, SliderElement::get_direction)
    }

    pub fn set_thumb_size(&self, app: &mut App, thumb_size: f64) {
        if let Some(slider) = app.try_get_as_mut::<SliderElement>(self.inner) {
            slider.set_thumb_size(thumb_size);
        }
    }

    pub fn thumb_size(&self, app: &App) -> f64 {
        app.try_get_as::<SliderElement>(self.inner)
            .map_or(0.0, SliderElement::get_thumb_size)
    }

    pub fn set_thumb_color(&self, app: &mut App, thumb_background_color: Brush) {
        if let Some(slider) = app.try_get_as_mut::<SliderElement>(self.inner) {
            slider.set_thumb_color(thumb_background_color);
        }
    }

    pub fn thumb_brush(&self, app: &App) -> Brush {
        app.try_get_as::<SliderElement>(self.inner)
            .map_or_else(Brush::default, SliderElement::get_thumb_brush)
    }

    pub fn set_thumb_border_radius(
        &self,
        app: &mut App,
        top: (f32, f32),
        right: (f32, f32),
        bottom: (f32, f32),
        left: (f32, f32),
    ) {
        if let Some(slider) = app.try_get_as_mut::<SliderElement>(self.inner) {
            slider.set_thumb_border_radius(top, right, bottom, left);
        }
    }

    pub fn thumb_border_radius(&self, app: &App) -> Option<[(f32, f32); 4]> {
        app.try_get_as::<SliderElement>(self.inner)
            .and_then(SliderElement::get_thumb_border_radius)
    }

    pub fn set_track_color(&self, app: &mut App, track_background_color: Color) {
        if let Some(slider) = app.try_get_as_mut::<SliderElement>(self.inner) {
            slider.set_track_brush(Brush::Color(track_background_color));
        }
    }

    pub fn set_track_gradient(&self, app: &mut App, track_background_gradient: Gradient) {
        if let Some(slider) = app.try_get_as_mut::<SliderElement>(self.inner) {
            slider.set_track_brush(Brush::Gradient(track_background_gradient));
        }
    }

    pub fn track_brush(&self, app: &App) -> Option<Brush> {
        app.try_get_as::<SliderElement>(self.inner)
            .and_then(SliderElement::get_track_brush)
    }

    pub fn set_track_border_radius(
        &self,
        app: &mut App,
        top: (f32, f32),
        right: (f32, f32),
        bottom: (f32, f32),
        left: (f32, f32),
    ) {
        if let Some(slider) = app.try_get_as_mut::<SliderElement>(self.inner) {
            slider.set_track_border_radius(top, right, bottom, left);
        }
    }

    pub fn track_border_radius(&self, app: &App) -> Option<[(f32, f32); 4]> {
        app.try_get_as::<SliderElement>(self.inner)
            .and_then(SliderElement::get_track_border_radius)
    }
}

impl SliderElement {
    pub fn create(
        elements: &mut RetainedElements,
        gummy_tree: &mut GummyTree,
        access_tree: &RetGuiAccessTree,
        by_internal_id: &mut ElementIds,
        thumb_size: f32,
    ) -> DynElement {
        let me = elements.insert_with(access_tree, by_internal_id, |me, access_tree| {
            Box::new(SliderElement {
                element_data: ElementData::new(me, false, access_tree),
                step: 1.0,
                min: 0.0,
                max: 100.0,
                direction: Default::default(),
                value: 0.0,
                dragging: false,
                thumb_size: thumb_size as f64,
                thumb_background_brush: Brush::Color(Color::BLACK),
                thumb_border_radius: None,
                track_background_brush: Some(Brush::Color(palette::css::DODGER_BLUE)),
                track_border_radius: None,
            })
        });

        {
            let element = elements.get_as_mut::<SliderElement>(me);
            element.element_data.create_layout_node(gummy_tree, None);
            element.element_data.set_accessibility_role(issho::Role::Slider);
            element.set_background_brush(Brush::Color(palette::css::LIGHT_GRAY));
            let border_radius = 25.0;
            element.set_border_radius(
                (border_radius, border_radius),
                (border_radius, border_radius),
                (border_radius, border_radius),
                (border_radius, border_radius),
            );
            if element.direction == SliderDirection::Horizontal {
                element.set_width(gummy_tree, Unit::Px(140.0));
                element.set_height(gummy_tree, Unit::Px(10.0));
            } else {
                element.set_height(gummy_tree, Unit::Px(140.0));
                element.set_width(gummy_tree, Unit::Px(10.0));
            }
        }
        me
    }

    pub fn set_value(&mut self, value: f64) {
        self.value = value.clamp(self.min, self.max);
        self.request_window_redraw();
    }

    fn update_value_from_event(&mut self, event_queue: &mut VecDeque<EventKind>, value: f64) {
        let value = value.clamp(self.min, self.max);
        if (value - self.value).abs() > f64::EPSILON {
            self.set_value(value);
            let target = self.element_data.me;
            event_queue.push_back(EventKind::SliderValueChanged(SliderValueChangedEvent::new(
                target, self.value,
            )));
        }
    }

    pub fn get_value(&self) -> f64 {
        self.value
    }

    /// Set the slider step value. Defaults to 1.
    pub fn set_step(&mut self, value: f64) {
        self.step = value;
    }

    pub fn get_step(&self) -> f64 {
        self.step
    }

    /// Set the minimum slider value. Defaults to 0.
    pub fn set_min(&mut self, min: f64) {
        self.min = min;
        self.value = self.value.clamp(self.min, self.max);
        self.request_window_redraw();
    }

    pub fn get_min(&self) -> f64 {
        self.min
    }

    /// Set the max slider value. Defaults to 100.
    pub fn set_max(&mut self, max: f64) {
        self.max = max;
        self.value = self.value.clamp(self.min, self.max);
        self.request_window_redraw();
    }

    pub fn get_max(&self) -> f64 {
        self.max
    }

    /// Set the slider direction.
    pub fn set_direction(&mut self, direction: SliderDirection) {
        self.direction = direction;
        self.request_window_redraw();
    }

    pub fn get_direction(&self) -> SliderDirection {
        self.direction
    }

    pub fn set_thumb_size(&mut self, thumb_size: f64) {
        self.thumb_size = thumb_size;
        self.request_window_redraw();
    }

    pub fn get_thumb_size(&self) -> f64 {
        self.thumb_size
    }

    pub fn set_thumb_color(&mut self, thumb_background_brush: Brush) {
        self.thumb_background_brush = thumb_background_brush;
        self.request_window_redraw();
    }

    pub fn get_thumb_brush(&self) -> Brush {
        self.thumb_background_brush.clone()
    }

    pub fn set_thumb_border_radius(
        &mut self,
        top: (f32, f32),
        right: (f32, f32),
        bottom: (f32, f32),
        left: (f32, f32),
    ) {
        self.thumb_border_radius = Some([top, right, bottom, left]);
        self.request_window_redraw();
    }

    pub fn get_thumb_border_radius(&self) -> Option<[(f32, f32); 4]> {
        self.thumb_border_radius
    }

    pub fn set_track_brush(&mut self, track_background_brush: Brush) {
        self.track_background_brush = Some(track_background_brush);
        self.request_window_redraw();
    }

    pub fn get_track_brush(&self) -> Option<Brush> {
        self.track_background_brush.clone()
    }

    pub fn set_track_border_radius(
        &mut self,
        top: (f32, f32),
        right: (f32, f32),
        bottom: (f32, f32),
        left: (f32, f32),
    ) {
        self.track_border_radius = Some([top, right, bottom, left]);
        self.request_window_redraw();
    }

    pub fn get_track_border_radius(&self) -> Option<[(f32, f32); 4]> {
        self.track_border_radius
    }

    pub(super) fn draw_track(&self, renderer: &mut dyn Renderer, scale_factor: f64) {
        if let Some(track_color) = self.get_track_brush() {
            let mut track_box = self.element_data().layout.local_box();

            let computed_element_rect = track_box.border_rectangle();

            let range = self.get_max() - self.get_min();
            let normalized_value = if range == 0.0 {
                0.0f32
            } else {
                ((self.get_value() - self.get_min()) / range) as f32
            };

            if self.get_direction() == SliderDirection::Horizontal {
                track_box.size.width = normalized_value * computed_element_rect.width;
            } else {
                track_box.size.height = normalized_value * computed_element_rect.height;

                track_box.position.y = computed_element_rect.bottom() as f64 - track_box.size.height as f64;
            }

            // Use the specified border radius or default to the slider's border radius.
            let track_radii = if let Some(br) = self.get_track_border_radius() {
                border_radius_to_vec_radius(br)
            } else {
                border_radius_to_vec_radius(self.style().get_border_radius())
            };

            let css_rounded_rect = CssRoundedRect::new(
                track_box.border_rectangle().to_kurbo(),
                [0.0, 0.0, 0.0, 0.0],
                track_radii,
            );
            let mut computed_border_spec = CssComputedBorder::new(css_rounded_rect);
            computed_border_spec.scale(scale_factor);

            let color_rect = TrblRectangle::new_all(Color::WHITE).to_array();
            draw_borders_generic(renderer, &computed_border_spec, color_rect, track_color);
        }
    }

    pub(super) fn draw_thumb(&self, renderer: &mut dyn Renderer, scale_factor: f64) {
        let thumb_pos = self.local_thumb_position(self.get_value());
        let thumb_size = self.get_thumb_size();
        let thumb_background_color = self.get_thumb_brush();
        let thumb_rect = Rectangle::new(
            thumb_pos.x as f32,
            thumb_pos.y as f32,
            thumb_size as f32,
            thumb_size as f32,
        );

        // Use the specified border radius or default to 50% (a circle).
        let thumb_radii = if let Some(br) = self.get_thumb_border_radius() {
            border_radius_to_vec_radius(br)
        } else {
            let half_size = thumb_size / 2.0;
            let half_size = Vec2::new(half_size, half_size);
            [half_size, half_size, half_size, half_size]
        };

        let css_rounded_rect = CssRoundedRect::new(thumb_rect.to_kurbo(), [0.0, 0.0, 0.0, 0.0], thumb_radii);
        let mut computed_border_spec = CssComputedBorder::new(css_rounded_rect);
        computed_border_spec.scale(scale_factor);
        let color_rect = TrblRectangle::new_all(Color::WHITE).to_array();
        draw_borders_generic(renderer, &computed_border_spec, color_rect, thumb_background_color);
    }

    pub(super) fn compute_step(&self, by: i32, current_value: f64) -> f64 {
        let delta = by.abs() as f64 * self.get_step();

        let value = if by > 0 {
            current_value + delta
        } else {
            current_value - delta
        };

        value.clamp(self.get_min(), self.get_max())
    }

    pub(super) fn compute_slider_value(&self, pointer_position: &Point) -> f64 {
        let content_rectangle = self.element_data().layout.world_box().content_rectangle();
        let start = if self.get_direction() == SliderDirection::Horizontal {
            content_rectangle.left() as f64
        } else {
            content_rectangle.bottom() as f64
        };
        let end = if self.get_direction() == SliderDirection::Horizontal {
            content_rectangle.right() as f64
        } else {
            content_rectangle.top() as f64
        };

        let pointer_position_component = if self.get_direction() == SliderDirection::Horizontal {
            pointer_position.x
        } else {
            pointer_position.y
        };

        let track_length = end - start;
        if track_length == 0.0 {
            return self.get_min();
        }

        // [0, 1]
        let normalized_value = ((pointer_position_component - start) / track_length).clamp(0.0, 1.0);
        let range = self.get_max() - self.get_min();
        let raw = self.get_min() + normalized_value * range;

        // Round the value to the nearest step.
        let stepped = self.get_min() + ((raw - self.get_min()) / self.get_step()).round() * self.get_step();
        stepped.clamp(self.get_min(), self.get_max())
    }

    pub(super) fn thumb_position(&self, thumb_value: f64) -> Point {
        self.thumb_position_in_rect(thumb_value, self.element_data().layout.world_box().content_rectangle())
    }

    pub(super) fn local_thumb_position(&self, thumb_value: f64) -> Point {
        self.thumb_position_in_rect(thumb_value, self.element_data().layout.computed_box.content_rectangle())
    }

    fn thumb_position_in_rect(
        &self,
        thumb_value: f64,
        content_rectangle: retgui_primitives::geometry::Rectangle,
    ) -> Point {
        let range = self.get_max() - self.get_min();
        let normalized_value = if range == 0.0 {
            0.0
        } else {
            ((thumb_value - self.get_min()) / range).clamp(0.0, 1.0)
        };

        let value = if self.get_direction() == SliderDirection::Horizontal {
            normalized_value * content_rectangle.width as f64
        } else {
            normalized_value * content_rectangle.height as f64
        };

        let thumb_offset = self.get_thumb_size() as f32 / 2.0;
        let x = if self.get_direction() == SliderDirection::Horizontal {
            f32::clamp(
                content_rectangle.left() + value as f32 - thumb_offset,
                content_rectangle.left(),
                content_rectangle.right() - self.get_thumb_size() as f32,
            )
        } else {
            content_rectangle.left() - thumb_offset + content_rectangle.width / 2.0
        };

        let y = if self.get_direction() == SliderDirection::Horizontal {
            content_rectangle.top() + content_rectangle.height / 2.0 - thumb_offset
        } else {
            f32::clamp(
                content_rectangle.bottom() - value as f32 - thumb_offset,
                content_rectangle.top(),
                content_rectangle.bottom() - self.get_thumb_size() as f32,
            )
        };

        Point::new(x as f64, y as f64)
    }
}

impl Element for Slider {
    fn as_dyn_element(&self) -> DynElement {
        self.inner
    }
}

impl crate::elements::HasElementData for SliderElement {
    fn element_data(&self) -> &ElementData {
        &self.element_data
    }

    fn element_data_mut(&mut self) -> &mut ElementData {
        &mut self.element_data
    }
}

impl ElementInternals for SliderElement {
    fn deep_clone(
        &self,
        elements: &mut RetainedElements,
        gummy_tree: &mut GummyTree,
        access_tree: &RetGuiAccessTree,
        by_internal_id: &mut ElementIds,
    ) -> DynElement {
        DynElement::new(clone_element::<Self, _>(
            self,
            elements,
            gummy_tree,
            access_tree,
            by_internal_id,
            |_, _| None,
        ))
    }

    fn apply_layout(
        &mut self,
        gummy_tree: &mut GummyTree,
        z_index: &mut u32,
        _text_context: &mut TextContext,
        scale_factor: f64,
    ) {
        let node = self.element_data.layout.gummy_node_id.unwrap();
        let layout = gummy_tree.get_layout(node);
        let has_new_layout = gummy_tree.has_new_layout(node);

        self.element_data.layout.has_new_layout.set(has_new_layout);

        if has_new_layout {
            self.resolve_box(layout, z_index);
            self.apply_borders(scale_factor);
            gummy_tree.mark_seen(node);
        }
    }

    fn draw(
        &self,
        _elements: &RetainedElements,
        _states: &ElementStates,
        _renderer: &mut dyn Renderer,
        _resource_manager: Arc<ResourceManager>,
        _scale_factor: f64,
        _text_context: &mut TextContext,
    ) {
        if !self.is_visible() {
            return;
        }

        self.maybe_start_overlay(_renderer);

        self.add_hit_testable(_renderer, true, _scale_factor);

        self.draw_borders(_renderer, _scale_factor);
        self.draw_track(_renderer, _scale_factor);
        self.draw_thumb(_renderer, _scale_factor);

        self.maybe_end_overlay(_renderer);
    }

    fn add_hit_testable(&self, renderer: &mut dyn Renderer, hit_testable: bool, scale_factor: f64) {
        if !hit_testable {
            return;
        }

        let track = self.element_data.layout.local_box().border_rectangle();
        let thumb_position = self.local_thumb_position(self.value);
        let thumb = Rectangle::new(
            thumb_position.x as f32,
            thumb_position.y as f32,
            self.thumb_size as f32,
            self.thumb_size as f32,
        );
        let left = track.left().min(thumb.left());
        let top = track.top().min(thumb.top());
        let right = track.right().max(thumb.right());
        let bottom = track.bottom().max(thumb.bottom());
        renderer.push_hit_testable(
            self.element_data.internal_id,
            Rectangle::new(left, top, right - left, bottom - top).scale(scale_factor),
        );
    }

    fn on_event(
        &mut self,
        elements: &mut RetainedElements,
        _gummy_tree: &mut GummyTree,
        _access_tree: &RetGuiAccessTree,
        _by_internal_id: &mut ElementIds,
        event_queue: &mut VecDeque<EventKind>,
        focus: &mut Option<DynElement>,
        focus_outline_visible: bool,
        _pending_animation_updates: &mut Vec<(DynElement, bool)>,
        _states: &mut ElementStates,
        event: &mut EventKind,
        _text_context: &mut TextContext,
    ) {
        match event {
            EventKind::KeyDown(key) | EventKind::KeyUp(key) => {
                if key.state != ElementState::Pressed || !self.is_focused() {
                    return;
                }

                let new_value = match key.code {
                    KeyCode::ArrowUp | KeyCode::ArrowRight => Some(self.compute_step(1, self.value)),
                    KeyCode::ArrowDown | KeyCode::ArrowLeft => Some(self.compute_step(-1, self.value)),
                    KeyCode::Home => Some(self.min),
                    KeyCode::End => Some(self.max),
                    KeyCode::PageUp => Some(self.compute_step(10, self.value)),
                    KeyCode::PageDown => Some(self.compute_step(-10, self.value)),
                    _ => None,
                };

                if let Some(new_value) = new_value {
                    self.update_value_from_event(event_queue, new_value);
                    key.stop_propagation();
                    key.prevent_default();
                }
            }
            EventKind::PointerUp(pointer_button_update) => {
                self.focus(elements, event_queue, focus, focus_outline_visible);
                self.dragging = false;
                self.release_pointer_capture(elements, pointer_button_update.pointer.pointer_id.unwrap());

                let value = self.compute_slider_value(&pointer_button_update.state.logical_point());
                self.update_value_from_event(event_queue, value);
            }
            EventKind::PointerDown(pointer_button_update) => {
                self.dragging = true;
                self.set_pointer_capture(elements, pointer_button_update.pointer.pointer_id.unwrap());

                let value = self.compute_slider_value(&pointer_button_update.state.logical_point());
                self.update_value_from_event(event_queue, value);
            }
            EventKind::PointerMoved(pointer_update) => {
                if !self.dragging {
                    return;
                }

                let value = self.compute_slider_value(&pointer_update.current.logical_point());
                self.update_value_from_event(event_queue, value);
            }
            _ => {}
        }

        //println!("Slider Value: {}", self.value);
    }

    fn in_bounds(&self, point: Point) -> bool {
        let element_data = &self.element_data;
        let rect = element_data.layout.world_box().border_rectangle();

        let thumb_pos = self.thumb_position(self.get_value());
        let thumb_size = self.get_thumb_size();
        let thumb_rect = Rectangle::new(
            thumb_pos.x as f32,
            thumb_pos.y as f32,
            thumb_size as f32,
            thumb_size as f32,
        );

        let contains = thumb_rect.contains(&point) || rect.contains(&point);
        contains
            && element_data
                .layout
                .clip_bounds
                .get()
                .is_none_or(|clip| clip.contains(&point))
    }
}

fn border_radius_to_vec_radius(border_radius: [(f32, f32); 4]) -> [Vec2; 4] {
    let br = border_radius;
    [
        Vec2::new(br[0].0 as f64, br[0].1 as f64),
        Vec2::new(br[1].0 as f64, br[1].1 as f64),
        Vec2::new(br[2].0 as f64, br[2].1 as f64),
        Vec2::new(br[3].0 as f64, br[3].1 as f64),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::App;
    use crate::elements::Slider;

    #[test]
    fn steps_one() {
        let mut app = App::new();
        let slider = Slider::new(&mut app, 16.0);
        let slider = app.get_as_mut::<SliderElement>(slider.inner);

        slider.set_value(50.0);
        let next_step = slider.compute_step(1, slider.get_value());

        assert_eq!(next_step as i32, 51i32);
    }

    #[test]
    fn steps_down_one() {
        let mut app = App::new();
        let slider = Slider::new(&mut app, 16.0);
        let slider = app.get_as_mut::<SliderElement>(slider.inner);

        slider.set_value(50.0);
        let next_step = slider.compute_step(-1, slider.get_value());

        assert_eq!(next_step as i32, 49i32);
    }
}
