use crate::components::Props;
use crate::components::Event;
use crate::elements::element::Element;
use crate::elements::element_data::ElementData;
use crate::elements::element_styles::ElementStyles;
use crate::layout::layout_context::LayoutContext;
use crate::elements::thumb::Thumb;
use crate::events::CraftMessage;
use crate::geometry::{Point, Rectangle};
use crate::reactive::element_state_store::{ElementStateStore, ElementStateStoreItem};
use crate::renderer::renderer::RenderList;
use crate::style::{Display, Style, Unit};
use crate::ComponentSpecification;
use crate::{generate_component_methods_no_children, palette};
use std::any::Any;
use std::sync::Arc;
use taffy::{NodeId, TaffyTree};
use winit::window::Window;
use crate::text::text_context::TextContext;

/// An element that represents an on or off state.
#[derive(Clone)]
pub struct Switch {
    pub element_data: ElementData,
    /// When `state.toggled` is None, use this as the default value.
    default_toggled: bool,

    /// A pseudo thumb, this is not stored in the user tree nor will it receive events.
    /// This is mostly for convenience, so that we can change the location and render it in the switch track container.
    thumb: Thumb,

    /// The style of the container/track when the switch is toggled. This style will get merged with the default style + user style.
    pub(crate) toggled_track_style: Style,

    /// The padding around the thumb and the track in pixels.
    spacing: f32,
    rounded: bool,
}

#[derive(Clone, Default)]
pub struct SwitchState {
    pub(crate) toggled: Option<bool>,
}

impl Element for Switch {
    fn element_data(&self) -> &ElementData {
        &self.element_data
    }

    fn element_data_mut(&mut self) -> &mut ElementData {
        &mut self.element_data
    }

    fn name(&self) -> &'static str {
        "Switch"
    }

    fn draw(
        &mut self,
        renderer: &mut RenderList,
        text_context: &mut TextContext,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        _root_node: NodeId,
        element_state: &mut ElementStateStore,
        pointer: Option<Point>,
        window: Option<Arc<dyn Window>>,
    ) {
        if !self.element_data.style.visible() {
            return;
        }
        self.draw_borders(renderer, element_state);
        self.thumb.pseudo_thumb.draw(renderer, text_context, taffy_tree, _root_node, element_state, pointer, window);
    }

    fn compute_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        element_state: &mut ElementStateStore,
        scale_factor: f64,
    ) -> Option<NodeId> {
        let state = self.get_state(element_state);
        self.merge_default_style();

        let default_toggled = self.default_toggled;
        let mut set_toggled_styles = || {
            self.element_data_mut().style = Style::merge(&self.element_data().style, &self.default_toggled_style());
            self.element_data_mut().style = Style::merge(&self.element_data().style, &self.toggled_track_style);
        };

        // Use the toggled styles when state.toggled is true or default_toggled is true.
        if let Some(toggled) = state.toggled {
            if toggled {
                set_toggled_styles();
            }
        } else if default_toggled {
            set_toggled_styles();
        }

        let child_node = self.thumb.compute_layout(taffy_tree, element_state, scale_factor, state.toggled.unwrap_or(default_toggled), self.rounded);

        let style: taffy::Style = self.element_data.style.to_taffy_style_with_scale_factor(scale_factor);
        self.element_data_mut().taffy_node_id = Some(taffy_tree.new_with_children(style, &[child_node]).unwrap());
        self.element_data().taffy_node_id
    }

    fn finalize_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        root_node: NodeId,
        position: Point,
        z_index: &mut u32,
        transform: glam::Mat4,
        element_state: &mut ElementStateStore,
        pointer: Option<Point>,
        text_context: &mut TextContext,
        clip_bounds: Option<Rectangle>,
    ) {
        let state = self.get_state(element_state);
        let result = taffy_tree.layout(root_node).unwrap();
        self.resolve_box(position, transform, result, z_index);
        self.resolve_clip(clip_bounds);
        self.finalize_borders(element_state);
        
        let x = if state.toggled.unwrap_or(self.default_toggled) {
            self.element_data.computed_box.content_rectangle().right() - self.spacing - self.thumb.size
        } else {
            self.element_data.computed_box.content_rectangle().left() + self.spacing
        };
        let y = self.element_data.computed_box.content_rectangle().top() + self.spacing;
        
        self.thumb.finalize_layout(
            taffy_tree,
            Point::new(x, y),
            z_index,
            transform,
            element_state,
            pointer,
            text_context,
        );
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn on_event(
        &self,
        message: &CraftMessage,
        element_state: &mut ElementStateStore,
        _text_context: &mut TextContext,
        should_style: bool,
        event: &mut Event,
    ) {
        self.on_style_event(message, element_state, should_style, event);
        let base_state = self.get_base_state_mut(element_state);
        let state = base_state.data.as_mut().downcast_mut::<SwitchState>().unwrap();

        if message.clicked() {
            if let Some(toggled) = state.toggled {
                // Negate the current toggled bool.
                state.toggled = Some(!toggled);
            } else {
                // Negate the default toggled bool when `state.toggled` is None.
                state.toggled = Some(!self.default_toggled);
            }

            // Emit the SwitchToggled event with the new value of `state.toggled`.
            event.result_message(CraftMessage::SwitchToggled(state.toggled.unwrap()));
            event.prevent_propagate();
        }
    }

    fn initialize_state(&mut self, _scaling_factor: f64) -> ElementStateStoreItem {
        ElementStateStoreItem {
            base: Default::default(),
            data: Box::new(SwitchState::default()),
        }
    }

    fn default_style(&self) -> Style {
        let mut style = Style::default();

        let thumb_diameter = self.thumb.size;
        let padding = self.spacing;

        let width  = thumb_diameter * 2.25;
        let height = thumb_diameter + padding * 2.0;
        
        *style.display_mut() = Display::Flex;
        *style.width_mut() = Unit::Px(width);
        *style.min_width_mut() = Unit::Px(width);
        *style.height_mut() = Unit::Px(height);
        *style.min_height_mut() = Unit::Px(height);
        *style.background_mut() = palette::css::LIGHT_GRAY;

        if self.rounded {
            let rounding = self.thumb.size / 1.5;
            *style.border_radius_mut() = [(rounding, rounding), (rounding, rounding), (rounding, rounding), (rounding, rounding)];
        }
        
        style
    }
}

impl Default for Switch {
    fn default() -> Self {
        Self::new(26.0)
    }
}

impl Switch {

    /// Sets the padding around the thumb and the track in pixels.
    pub fn spacing(mut self, amount: f32) -> Self {
        self.spacing = amount;
        self
    }
    
    pub fn round(mut self) -> Self {
        self.rounded = true;
        self
    }
    
    fn default_toggled_style(&self) -> Style {
        let mut style = Style::default();
        *style.background_mut() = palette::css::DODGER_BLUE;
        Style::merge(&self.default_style(), &style)
    }

    pub fn thumb_style(mut self, thumb_style: Style) -> Self {
        self.thumb.thumb_style(thumb_style);
        self
    }

    pub fn toggled_thumb_style(mut self, toggled_thumb_style: Style) -> Self {
        self.thumb.toggled_thumb_style(toggled_thumb_style);
        self
    }

    pub fn toggled_style(mut self, toggled_thumb_style: Style) -> Self {
        self.toggled_track_style = toggled_thumb_style;
        self
    }

    pub fn default_toggled(mut self, default_toggled: bool) -> Self {
        self.default_toggled = default_toggled;
        self
    }

    #[allow(dead_code)]
    fn get_state<'a>(&self, element_state: &'a ElementStateStore) -> &'a SwitchState {
        element_state.storage.get(&self.element_data.component_id).unwrap().data.as_ref().downcast_ref().unwrap()
    }

    pub fn new(size: f32) -> Switch {
        Switch {
            element_data: Default::default(),
            default_toggled: false,
            thumb: Thumb {
                pseudo_thumb: Default::default(),
                toggled_thumb_style: Default::default(),
                size,
            },
            toggled_track_style: Default::default(),
            spacing: 4.0,
            rounded: false,
        }
    }

    generate_component_methods_no_children!();
}

impl ElementStyles for Switch {
    fn styles_mut(&mut self) -> &mut Style {
        self.element_data.current_style_mut()
    }
}
