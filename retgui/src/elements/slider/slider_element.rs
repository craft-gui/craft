use peniko::Color;
use retgui_primitives::geometry::{Point, Rectangle};
use std::sync::Arc;

use retgui_primitives::brush::Brush;
use retgui_primitives::gradient::Gradient;

use retgui_renderer::renderer::Renderer;

use retgui_resource_manager::ResourceManager;

use winit::event::ElementState;
use winit::keyboard::KeyCode;

use crate::elements::element_data::ElementData;
use crate::elements::traits::clone_element;
use crate::elements::{DynElement, Element, ElementNode, Elements};
use crate::events::{Event, EventKind, SliderValueChangedEvent};
use crate::layout::GummyTree;
use crate::palette;
use crate::style::Unit;
use crate::text::text_context::TextContext;

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
pub(crate) struct SliderNode {
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
    pub fn new(elements: &mut Elements, thumb_size: f32) -> Self {
        Self {
            inner: SliderNode::create(elements, thumb_size),
        }
    }

    pub fn set_value(&self, elements: &mut Elements, value: f64) {
        if let Some(slider) = elements.try_get_as_mut::<SliderNode>(self.inner) {
            slider.set_value(value);
        }
    }

    pub fn value(&self, elements: &Elements) -> f64 {
        elements
            .try_get_as::<SliderNode>(self.inner)
            .map_or(0.0, SliderNode::get_value)
    }

    pub fn set_step(&self, elements: &mut Elements, value: f64) {
        if let Some(slider) = elements.try_get_as_mut::<SliderNode>(self.inner) {
            slider.set_step(value);
        }
    }

    pub fn step(&self, elements: &Elements) -> f64 {
        elements
            .try_get_as::<SliderNode>(self.inner)
            .map_or(0.0, SliderNode::get_step)
    }

    pub fn set_min(&self, elements: &mut Elements, min: f64) {
        if let Some(slider) = elements.try_get_as_mut::<SliderNode>(self.inner) {
            slider.set_min(min);
        }
    }

    pub fn min(&self, elements: &Elements) -> f64 {
        elements
            .try_get_as::<SliderNode>(self.inner)
            .map_or(0.0, SliderNode::get_min)
    }

    pub fn set_max(&self, elements: &mut Elements, max: f64) {
        if let Some(slider) = elements.try_get_as_mut::<SliderNode>(self.inner) {
            slider.set_max(max);
        }
    }

    pub fn max(&self, elements: &Elements) -> f64 {
        elements
            .try_get_as::<SliderNode>(self.inner)
            .map_or(0.0, SliderNode::get_max)
    }

    pub fn set_direction(&self, elements: &mut Elements, direction: SliderDirection) {
        if let Some(slider) = elements.try_get_as_mut::<SliderNode>(self.inner) {
            slider.set_direction(direction);
        }
    }

    pub fn direction(&self, elements: &Elements) -> SliderDirection {
        elements
            .try_get_as::<SliderNode>(self.inner)
            .map_or_else(SliderDirection::default, SliderNode::get_direction)
    }

    pub fn set_thumb_size(&self, elements: &mut Elements, thumb_size: f64) {
        if let Some(slider) = elements.try_get_as_mut::<SliderNode>(self.inner) {
            slider.set_thumb_size(thumb_size);
        }
    }

    pub fn thumb_size(&self, elements: &Elements) -> f64 {
        elements
            .try_get_as::<SliderNode>(self.inner)
            .map_or(0.0, SliderNode::get_thumb_size)
    }

    pub fn set_thumb_color(&self, elements: &mut Elements, thumb_background_color: Brush) {
        if let Some(slider) = elements.try_get_as_mut::<SliderNode>(self.inner) {
            slider.set_thumb_color(thumb_background_color);
        }
    }

    pub fn thumb_brush(&self, elements: &Elements) -> Brush {
        elements
            .try_get_as::<SliderNode>(self.inner)
            .map_or_else(Brush::default, SliderNode::get_thumb_brush)
    }

    pub fn set_thumb_border_radius(
        &self,
        elements: &mut Elements,
        top: (f32, f32),
        right: (f32, f32),
        bottom: (f32, f32),
        left: (f32, f32),
    ) {
        if let Some(slider) = elements.try_get_as_mut::<SliderNode>(self.inner) {
            slider.set_thumb_border_radius(top, right, bottom, left);
        }
    }

    pub fn thumb_border_radius(&self, elements: &Elements) -> Option<[(f32, f32); 4]> {
        elements
            .try_get_as::<SliderNode>(self.inner)
            .and_then(SliderNode::get_thumb_border_radius)
    }

    pub fn set_track_color(&self, elements: &mut Elements, track_background_color: Color) {
        if let Some(slider) = elements.try_get_as_mut::<SliderNode>(self.inner) {
            slider.set_track_brush(Brush::Color(track_background_color));
        }
    }

    pub fn set_track_gradient(&self, elements: &mut Elements, track_background_gradient: Gradient) {
        if let Some(slider) = elements.try_get_as_mut::<SliderNode>(self.inner) {
            slider.set_track_brush(Brush::Gradient(track_background_gradient));
        }
    }

    pub fn track_brush(&self, elements: &Elements) -> Option<Brush> {
        elements
            .try_get_as::<SliderNode>(self.inner)
            .and_then(SliderNode::get_track_brush)
    }

    pub fn set_track_border_radius(
        &self,
        elements: &mut Elements,
        top: (f32, f32),
        right: (f32, f32),
        bottom: (f32, f32),
        left: (f32, f32),
    ) {
        if let Some(slider) = elements.try_get_as_mut::<SliderNode>(self.inner) {
            slider.set_track_border_radius(top, right, bottom, left);
        }
    }

    pub fn track_border_radius(&self, elements: &Elements) -> Option<[(f32, f32); 4]> {
        elements
            .try_get_as::<SliderNode>(self.inner)
            .and_then(SliderNode::get_track_border_radius)
    }
}

impl SliderNode {
    pub fn create(elements: &mut Elements, thumb_size: f32) -> DynElement {
        let me = elements.insert_with(|me, access_tree| {
            Box::new(Self {
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

        elements.create_layout_node(me, None);
        elements.with_gummy_tree(|gummy_tree, elements| {
            let element = elements.get_as_mut::<Self>(me);
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
        });
        me
    }

    pub fn set_value(&mut self, value: f64) {
        self.value = value.clamp(self.min, self.max);
        self.request_window_redraw();
    }

    fn update_value_from_event(&mut self, elements: &mut Elements, value: f64) {
        let value = value.clamp(self.min, self.max);
        if (value - self.value).abs() > f64::EPSILON {
            self.set_value(value);
            let target = self.element_data.me;
            elements.queue_event(EventKind::SliderValueChanged(SliderValueChangedEvent::new(
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
}

impl Element for Slider {
    fn as_dyn_element(&self) -> DynElement {
        self.inner
    }
}

impl crate::elements::ElementNodeData for SliderNode {
    fn element_data(&self) -> &ElementData {
        &self.element_data
    }

    fn element_data_mut(&mut self) -> &mut ElementData {
        &mut self.element_data
    }
}

impl ElementNode for SliderNode {
    fn deep_clone(&self, elements: &mut Elements) -> DynElement {
        DynElement::new(clone_element::<Self, _>(self, elements, |_, _| None))
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
        _elements: &Elements,
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

    fn on_event(&mut self, elements: &mut Elements, event: &mut EventKind, _text_context: &mut TextContext) {
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
                    self.update_value_from_event(elements, new_value);
                    key.stop_propagation();
                    key.prevent_default();
                }
            }
            EventKind::PointerUp(pointer_button_update) => {
                self.focus(elements);
                self.dragging = false;
                self.release_pointer_capture(elements, pointer_button_update.pointer.pointer_id.unwrap());

                let value = self.compute_slider_value(&pointer_button_update.state.logical_point());
                self.update_value_from_event(elements, value);
            }
            EventKind::PointerDown(pointer_button_update) => {
                self.dragging = true;
                self.set_pointer_capture(elements, pointer_button_update.pointer.pointer_id.unwrap());

                let value = self.compute_slider_value(&pointer_button_update.state.logical_point());
                self.update_value_from_event(elements, value);
            }
            EventKind::PointerMoved(pointer_update) => {
                if !self.dragging {
                    return;
                }

                let value = self.compute_slider_value(&pointer_update.current.logical_point());
                self.update_value_from_event(elements, value);
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
