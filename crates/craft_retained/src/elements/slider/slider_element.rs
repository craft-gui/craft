use std::any::Any;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

use craft_primitives::geometry::Rectangle;
use craft_renderer::RenderList;
use kurbo::{Affine, Point};
use peniko::Color;
use ui_events::keyboard::{Code, KeyState};
use ui_events::pointer::PointerId;

use crate::app::queue_event;
use crate::elements::core::ElementInternals;
use crate::elements::element::AsElement;
use crate::elements::element_data::ElementData;
use crate::elements::{Element, ElementImpl};
use crate::events::{CraftMessage, Event};
use crate::layout::TaffyTree;
use crate::palette;
use crate::style::Unit;
use crate::text::text_context::TextContext;

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub enum SliderDirection {
    #[default]
    Horizontal,
    Vertical,
}

#[derive(Clone)]
pub struct Slider {
    pub inner: Rc<RefCell<SliderInner>>,
}

pub struct SliderInner {
    element_data: ElementData,

    step: f64,
    min: f64,
    max: f64,
    direction: SliderDirection,
    value: f64,
    dragging: bool,

    // Thumb
    thumb_size: f64,
    thumb_background_color: Color,
    thumb_border_radius: Option<[(f32, f32); 4]>,

    // Track
    track_background_color: Option<Color>,
    track_border_radius: Option<[(f32, f32); 4]>,
}

impl Slider {
    pub fn new(thumb_size: f32) -> Self {
        Self {
            inner: SliderInner::new(thumb_size),
        }
    }

    pub fn value(self, value: f64) -> Self {
        self.inner.borrow_mut().set_value(value);
        self
    }

    pub fn get_value(&self) -> f64 {
        self.inner.borrow().get_value()
    }

    pub fn step(self, value: f64) -> Self {
        self.inner.borrow_mut().set_step(value);
        self
    }

    pub fn get_step(&self) -> f64 {
        self.inner.borrow().get_step()
    }

    pub fn min(self, min: f64) -> Self {
        self.inner.borrow_mut().set_min(min);
        self
    }

    pub fn get_min(&self) -> f64 {
        self.inner.borrow().get_min()
    }

    pub fn max(self, max: f64) -> Self {
        self.inner.borrow_mut().set_max(max);
        self
    }

    pub fn get_max(&self) -> f64 {
        self.inner.borrow().get_max()
    }

    pub fn direction(self, direction: SliderDirection) -> Self {
        self.inner.borrow_mut().set_direction(direction);
        self
    }

    pub fn get_direction(&self) -> SliderDirection {
        self.inner.borrow().get_direction()
    }

    pub fn thumb_size(self, thumb_size: f64) -> Self {
        self.inner.borrow_mut().set_thumb_size(thumb_size);
        self
    }

    pub fn get_thumb_size(&self) -> f64 {
        self.inner.borrow().get_thumb_size()
    }

    pub fn thumb_color(self, thumb_background_color: Color) -> Self {
        self.inner.borrow_mut().set_thumb_color(thumb_background_color);
        self
    }

    pub fn get_thumb_color(&self) -> Color {
        self.inner.borrow().get_thumb_color()
    }

    pub fn thumb_border_radius(self, top: (f32, f32), right: (f32, f32), bottom: (f32, f32), left: (f32, f32)) -> Self {
        self.inner
            .borrow_mut()
            .set_thumb_border_radius(top, right, bottom, left);
        self
    }

    pub fn get_thumb_border_radius(&self) -> Option<[(f32, f32); 4]> {
        self.inner.borrow().get_thumb_border_radius()
    }

    pub fn track_color(self, track_background_color: Color) -> Self {
        self.inner.borrow_mut().set_track_color(track_background_color);
        self
    }

    pub fn get_track_color(&self) -> Option<Color> {
        self.inner.borrow().get_track_color()
    }

    pub fn track_border_radius(self, top: (f32, f32), right: (f32, f32), bottom: (f32, f32), left: (f32, f32)) -> Self {
        self.inner
            .borrow_mut()
            .set_track_border_radius(top, right, bottom, left);
        self
    }

    pub fn get_track_border_radius(&self) -> Option<[(f32, f32); 4]> {
        self.inner.borrow().get_track_border_radius()
    }
}

impl SliderInner {
    pub fn new(thumb_size: f32) -> Rc<RefCell<Self>> {
        let me = Rc::new_cyclic(|me: &Weak<RefCell<Self>>| {
            RefCell::new(Self {
                element_data: ElementData::new(me.clone(), false),
                step: 1.0,
                min: 0.0,
                max: 100.0,
                direction: Default::default(),
                value: 0.0,
                dragging: false,
                thumb_size: thumb_size as f64,
                thumb_background_color: Color::BLACK,
                thumb_border_radius: None,
                track_background_color: Some(palette::css::DODGER_BLUE),
                track_border_radius: None,
            })
        });

        me.borrow_mut().element_data.create_layout_node(None);

        me.borrow_mut().set_background_color(palette::css::LIGHT_GRAY);
        let border_radius = 25.0;
        me.borrow_mut().border_radius(
            (border_radius, border_radius),
            (border_radius, border_radius),
            (border_radius, border_radius),
            (border_radius, border_radius),
        );
        if me.borrow_mut().direction == SliderDirection::Horizontal {
            me.borrow_mut().set_width(Unit::Px(140.0));
            me.borrow_mut().set_height(Unit::Px(10.0));
        } else {
            me.borrow_mut().set_height(Unit::Px(140.0));
            me.borrow_mut().set_width(Unit::Px(10.0));
        }

        // TODO: FIX
        /* {
            let mut taffy_tree = self.element_data.taffy_tree.borrow_mut();
            let node_id = taffy_tree.new_leaf(me.borrow().style().to_taffy_style());
            me.borrow_mut().element_data.layout_item.taffy_node_id = Some(node_id);
        });

        ELEMENTS.with_borrow_mut(|elements| {
            elements.insert(me.borrow().deref());
        });*/

        me
    }

    pub fn set_value(&mut self, value: f64) {
        self.value = value;
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
    }

    pub fn get_min(&self) -> f64 {
        self.min
    }

    /// Set the max slider value. Defaults to 100.
    pub fn set_max(&mut self, max: f64) {
        self.max = max;
    }

    pub fn get_max(&self) -> f64 {
        self.max
    }

    /// Set the slider direction.
    pub fn set_direction(&mut self, direction: SliderDirection) {
        self.direction = direction;
    }

    pub fn get_direction(&self) -> SliderDirection {
        self.direction
    }

    pub fn set_thumb_size(&mut self, thumb_size: f64) {
        self.thumb_size = thumb_size;
    }

    pub fn get_thumb_size(&self) -> f64 {
        self.thumb_size
    }

    pub fn set_thumb_color(&mut self, thumb_background_color: Color) {
        self.thumb_background_color = thumb_background_color;
    }

    pub fn get_thumb_color(&self) -> Color {
        self.thumb_background_color
    }

    pub fn set_thumb_border_radius(
        &mut self,
        top: (f32, f32),
        right: (f32, f32),
        bottom: (f32, f32),
        left: (f32, f32),
    ) {
        self.thumb_border_radius = Some([top, right, bottom, left]);
    }

    pub fn get_thumb_border_radius(&self) -> Option<[(f32, f32); 4]> {
        self.thumb_border_radius
    }

    pub fn set_track_color(&mut self, track_background_color: Color) {
        self.track_background_color = Some(track_background_color);
    }

    pub fn get_track_color(&self) -> Option<Color> {
        self.track_background_color
    }

    pub fn set_track_border_radius(
        &mut self,
        top: (f32, f32),
        right: (f32, f32),
        bottom: (f32, f32),
        left: (f32, f32),
    ) {
        self.track_border_radius = Some([top, right, bottom, left]);
    }

    pub fn get_track_border_radius(&self) -> Option<[(f32, f32); 4]> {
        self.track_border_radius
    }
}

impl Element for Slider {}

impl AsElement for Slider {
    fn as_element_rc(&self) -> Rc<RefCell<dyn ElementImpl>> {
        self.inner.clone()
    }
}

impl crate::elements::core::ElementData for SliderInner {
    fn element_data(&self) -> &ElementData {
        &self.element_data
    }

    fn element_data_mut(&mut self) -> &mut ElementData {
        &mut self.element_data
    }
}

impl ElementImpl for SliderInner {
    fn in_bounds(&self, point: Point) -> bool {
        let element_data = &self.element_data;
        let rect = element_data.layout_item.computed_box_transformed.border_rectangle();

        let thumb_pos = self.thumb_position(self.get_value());
        let thumb_size = self.get_thumb_size();
        let thumb_rect = Rectangle::new(
            thumb_pos.x as f32,
            thumb_pos.y as f32,
            thumb_size as f32,
            thumb_size as f32,
        );

        if thumb_rect.contains(&point) {
            return true;
        }

        if let Some(clip) = element_data.layout_item.clip_bounds {
            match rect.intersection(&clip) {
                Some(bounds) => bounds.contains(&point),
                None => false,
            }
        } else {
            rect.contains(&point)
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl ElementInternals for SliderInner {
    fn apply_layout(
        &mut self,
        taffy_tree: &mut TaffyTree,
        position: Point,
        z_index: &mut u32,
        transform: Affine,
        _pointer: Option<Point>,
        _text_context: &mut TextContext,
        clip_bounds: Option<Rectangle>,
        scale_factor: f64,
    ) {
        let node = self.element_data.layout_item.taffy_node_id.unwrap();
        let layout = taffy_tree.layout(node);
        let has_new_layout = taffy_tree.get_has_new_layout(node);

        let dirty = has_new_layout
            || transform != self.element_data.layout_item.get_transform()
            || position != self.element_data.layout_item.position;
        self.element_data.layout_item.has_new_layout = has_new_layout;

        if dirty {
            self.resolve_box(position, transform, layout, z_index);

            self.apply_borders(scale_factor);
            self.apply_clip(clip_bounds);
        }
    }

    fn draw(
        &mut self,
        renderer: &mut RenderList,
        _text_context: &mut TextContext,
        _pointer: Option<Point>,
        scale_factor: f64,
    ) {
        if !self.is_visible() {
            return;
        }

        self.add_hit_testable(renderer, true, scale_factor);

        self.draw_borders(renderer, scale_factor);
        self.draw_track(renderer, scale_factor);
        self.draw_thumb(renderer);
    }

    fn on_event(
        &mut self,
        message: &CraftMessage,
        _text_context: &mut TextContext,
        event: &mut Event,
        _target: Option<Rc<RefCell<dyn ElementInternals>>>,
    ) {
        // @HARDCODED
        let focused = true;

        match message {
            CraftMessage::KeyboardInputEvent(key) => {
                if key.state != KeyState::Down || !focused {
                    return;
                }

                let new_value = match key.code {
                    Code::ArrowUp | Code::ArrowRight => Some(self.compute_step(1, self.value)),
                    Code::ArrowDown | Code::ArrowLeft => Some(self.compute_step(-1, self.value)),
                    Code::Home => Some(self.min),
                    Code::End => Some(self.max),
                    Code::PageUp => Some(self.compute_step(10, self.value)),
                    Code::PageDown => Some(self.compute_step(-10, self.value)),
                    _ => None,
                };

                if let Some(new_value) = new_value {
                    self.value = new_value;

                    let new_event = Event::new(event.target.clone());
                    queue_event(new_event, CraftMessage::SliderValueChanged(self.value));
                }
            }
            CraftMessage::PointerButtonUp(pointer_button_update) => {
                self.dragging = false;
                // FIXME: Turn pointer capture on with the correct device id.
                self.release_pointer_capture(PointerId::new(1).unwrap());

                let value = self.compute_slider_value(&Point::new(
                    pointer_button_update.state.position.x,
                    pointer_button_update.state.position.y,
                ));
                self.value = value;

                let new_event = Event::new(event.target.clone());
                queue_event(new_event, CraftMessage::SliderValueChanged(self.value));
            }
            CraftMessage::PointerButtonDown(pointer_button_update) => {
                self.dragging = true;
                // FIXME: Turn pointer capture on with the correct device id.
                self.set_pointer_capture(PointerId::new(1).unwrap());

                let value = self.compute_slider_value(&Point::new(
                    pointer_button_update.state.position.x,
                    pointer_button_update.state.position.y,
                ));
                self.value = value;

                let new_event = Event::new(event.target.clone());
                queue_event(new_event, CraftMessage::SliderValueChanged(self.value));
            }
            CraftMessage::PointerMovedEvent(pointer_update) => {
                if !self.dragging {
                    return;
                }

                let value = self.compute_slider_value(&Point::new(
                    pointer_update.current.position.x,
                    pointer_update.current.position.y,
                ));
                self.value = value;

                let new_event = Event::new(event.target.clone());
                queue_event(new_event, CraftMessage::SliderValueChanged(self.value));
            }
            _ => {}
        }

        //println!("Slider Value: {}", self.value);
    }
}
