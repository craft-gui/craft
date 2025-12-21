use crate::app::ELEMENTS;
use crate::app::TAFFY_TREE;
use crate::elements::core::ElementInternals;
use crate::elements::element_data::ElementData;
use crate::elements::Element;
use crate::events::{dispatch_event, CraftMessage, Event};
use crate::layout::layout_context::LayoutContext;
use crate::palette;
use crate::style::Unit;
use crate::text::text_context::TextContext;
use craft_primitives::geometry::Rectangle;
use craft_renderer::RenderList;
use kurbo::{Affine, Point};
use peniko::Color;
use std::any::Any;
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::{Rc, Weak};
use taffy::TaffyTree;
use ui_events::keyboard::{Code, KeyState};
use ui_events::pointer::PointerId;

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub enum SliderDirection {
    #[default]
    Horizontal,
    Vertical,
}

pub struct Slider {
    element_data: ElementData,
    me: Option<Weak<RefCell<Slider>>>,

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
    pub fn new(thumb_size: f32) -> Rc<RefCell<Self>> {
        let me = Rc::new(RefCell::new(Self {
            element_data: ElementData::new(true),
            me: None,
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
        }));

        me.borrow_mut().background_color(palette::css::LIGHT_GRAY);
        let border_radius = 25.0;
        me.borrow_mut().border_radius(
            (border_radius, border_radius),
            (border_radius, border_radius),
            (border_radius, border_radius),
            (border_radius, border_radius)
        );
        if me.borrow_mut().direction == SliderDirection::Horizontal {
            me.borrow_mut().width(Unit::Px(140.0));
            me.borrow_mut().height(Unit::Px(10.0));
        } else {
            me.borrow_mut().height(Unit::Px(140.0));
            me.borrow_mut().width(Unit::Px(10.0));
        }


        TAFFY_TREE.with_borrow_mut(|taffy_tree| {
            let node_id = taffy_tree.new_leaf(me.borrow().style().to_taffy_style()).expect("TODO: panic message");
            me.borrow_mut().element_data.layout_item.taffy_node_id = Some(node_id);
        });

        let me_element: Rc<RefCell<dyn Element>> = me.clone();
        me.borrow_mut().me = Some(Rc::downgrade(&me.clone()));
        me.borrow_mut().element_data.me = Some(Rc::downgrade(&me_element));

        ELEMENTS.with_borrow_mut(|elements| {
            elements.insert(me.borrow().deref());
        });

        me
    }

    pub fn value(&mut self, value: f64) -> &mut Self {
        self.value = value;
        self
    }

    pub fn get_value(&self) -> f64 {
        self.value
    }

    /// Set the slider step value. Defaults to 1.
    pub fn step(&mut self, value: f64) -> &mut Self {
        self.step = value;
        self
    }

    pub fn get_step(&self) -> f64 {
        self.step
    }

    /// Set the minimum slider value. Defaults to 0.
    pub fn min(&mut self, min: f64) -> &mut Self {
        self.min = min;
        self
    }

    pub fn get_min(&self) -> f64 {
        self.min
    }

    /// Set the max slider value. Defaults to 100.
    pub fn max(&mut self, max: f64) -> &mut Self {
        self.max = max;
        self
    }

    pub fn get_max(&self) -> f64 {
        self.max
    }

    /// Set the slider direction.
    pub fn direction(&mut self, direction: SliderDirection) -> &mut Self {
        self.direction = direction;
        self
    }

    pub fn get_direction(&self) -> SliderDirection {
        self.direction
    }

    pub fn thumb_size(&mut self, thumb_size: f64) -> &mut Self {
        self.thumb_size = thumb_size;
        self
    }

    pub fn get_thumb_size(&self) -> f64 {
        self.thumb_size
    }

    pub fn thumb_color(&mut self, thumb_background_color: Color) -> &mut Self {
        self.thumb_background_color = thumb_background_color;
        self
    }

    pub fn get_thumb_color(&self) -> Color {
        self.thumb_background_color
    }

    pub fn thumb_border_radius(&mut self, top: (f32, f32), right: (f32, f32), bottom: (f32, f32), left: (f32, f32)) -> &mut Self {
        self.thumb_border_radius = Some([top, right, bottom, left]);
        self
    }

    pub fn get_thumb_border_radius(&self) -> Option<[(f32, f32); 4]> {
        self.thumb_border_radius
    }

    pub fn track_color(&mut self, track_background_color: Color) -> &mut Self {
        self.track_background_color = Some(track_background_color);
        self
    }

    pub fn get_track_color(&self) -> Option<Color> {
        self.track_background_color
    }

    pub fn track_border_radius(&mut self, top: (f32, f32), right: (f32, f32), bottom: (f32, f32), left: (f32, f32)) -> &mut Self {
        self.track_border_radius = Some([top, right, bottom, left]);
        self
    }

    pub fn get_track_border_radius(&self) -> Option<[(f32, f32); 4]> {
        self.track_border_radius
    }

}

impl crate::elements::core::ElementData for Slider {
    fn element_data(&self) -> &ElementData {
        &self.element_data
    }

    fn element_data_mut(&mut self) -> &mut ElementData {
        &mut self.element_data
    }
}

impl Element for Slider {

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn in_bounds(&self, point: Point) -> bool {
        let element_data = &self.element_data;
        let rect = element_data.layout_item.computed_box_transformed.border_rectangle();

        let thumb_pos = self.thumb_position(self.get_value());
        let thumb_size = self.get_thumb_size();
        let thumb_rect = Rectangle::new(thumb_pos.x as f32, thumb_pos.y as f32, thumb_size as f32, thumb_size as f32);

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
}

impl ElementInternals for Slider {

    fn apply_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        position: Point,
        z_index: &mut u32,
        transform: Affine,
        _pointer: Option<Point>,
        _text_context: &mut TextContext,
        clip_bounds: Option<Rectangle>,
        scale_factor: f64,
    ) {
        let layout = taffy_tree.layout(self.element_data.layout_item.taffy_node_id.unwrap()).unwrap();
        self.resolve_box(position, transform, layout, z_index);

        self.apply_borders(scale_factor);
        self.apply_clip(clip_bounds);
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
        let focused = false;

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
                    _ => {
                        None
                    }
                };

                if let Some(new_value) = new_value {
                    self.value = new_value;
                    //event.result_message(CraftMessage::SliderValueChanged(self.value));
                }
            }
            CraftMessage::PointerButtonUp(pointer_button_update) => {
                self.dragging = false;
                // FIXME: Turn pointer capture on with the correct device id.
                self.release_pointer_capture(PointerId::new(1).unwrap());

                let value = self.compute_slider_value(&Point::new(pointer_button_update.state.position.x, pointer_button_update.state.position.y));
                self.value = value;
                //event.result_message(CraftMessage::SliderValueChanged(value));
            }
            CraftMessage::PointerButtonDown(pointer_button_update) => {
                self.dragging = true;
                // FIXME: Turn pointer capture on with the correct device id.
                self.set_pointer_capture(PointerId::new(1).unwrap());

                let value = self.compute_slider_value(&Point::new(pointer_button_update.state.position.x, pointer_button_update.state.position.y));
                self.value = value;
                //event.result_message(CraftMessage::SliderValueChanged(value));

                let new_event = Event::new(event.target.clone());
                dispatch_event(new_event, CraftMessage::SliderValueChanged(self.value));
            }
            CraftMessage::PointerMovedEvent(pointer_update) => {
                if !self.dragging {
                    return;
                }

                let value = self.compute_slider_value(&Point::new(pointer_update.current.position.x, pointer_update.current.position.y));
                self.value = value;
                //event.result_message(CraftMessage::SliderValueChanged(value));
            }
            _ => {}
        }

        //println!("Slider Value: {}", self.value);
    }
}
