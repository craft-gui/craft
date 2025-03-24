use crate::components::UpdateResult;
use crate::elements::common_element_data::CommonElementData;
use crate::elements::element::Element;
use crate::elements::element_styles::ElementStyles;
use crate::elements::layout_context::LayoutContext;
use crate::elements::Container;
use crate::events::OkuMessage;
use crate::geometry::Point;
use crate::reactive::element_state_store::{ElementStateStore, ElementStateStoreItem};
use crate::style::{AlignItems, Display, JustifyContent, Style, Unit};
use crate::{palette, RendererBox};
use cosmic_text::FontSystem;
use std::any::Any;
use std::sync::Arc;
use taffy::{NodeId, Position, TaffyTree};
use winit::window::Window;

/// An element that represents an on or off state.
#[derive(Clone, Debug)]
pub struct Switch {
    pub common_element_data: CommonElementData,
    /// When `state.toggled` is None, use this as the default value.
    default_toggled: bool,
    /// A pseudo thumb element, this is not stored in the user tree nor will it receive events.
    /// This is mostly for convenience, so that we can change the location and render it in the switch track container. 
    pseudo_thumb: Container,
    /// The style of the container/track when the switch is toggled. This style will get merged with the default style + user style.
    toggled_track_style: Style,
    /// The style of the thumb when the switch is toggled. This style will get merged with the default style + user style.
    toggled_thumb_style: Style,
}

#[derive(Clone, Default)]
pub struct SwitchState {
    pub(crate) toggled: Option<bool>,
}

impl Element for Switch {
    fn common_element_data(&self) -> &CommonElementData {
        &self.common_element_data
    }

    fn common_element_data_mut(&mut self) -> &mut CommonElementData {
        &mut self.common_element_data
    }

    fn name(&self) -> &'static str {
        "Switch"
    }

    fn draw(
        &mut self,
        renderer: &mut RendererBox,
        font_system: &mut FontSystem,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        _root_node: NodeId,
        element_state: &mut ElementStateStore,
        pointer: Option<Point>,
        window: Option<Arc<dyn Window>>
    ) {
        self.draw_borders(renderer);
        self.pseudo_thumb.draw(renderer, font_system, taffy_tree, _root_node, element_state, pointer, window);
    }

    fn compute_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        element_state: &mut ElementStateStore,
        scale_factor: f64,
    ) -> Option<NodeId> {
        let state = self.get_state(element_state);
        self.merge_default_style();

        // FIXME: Use insets and position absolute after we fix a few Taffy bugs.
        self.pseudo_thumb.common_element_data_mut().style = Style::merge(&self.default_thumb_style(), &self.pseudo_thumb.common_element_data_mut().style);
        
        let default_toggled = self.default_toggled;
        let mut set_toggled_styles = || {
            self.common_element_data_mut().style = Style::merge(&self.common_element_data().style, &self.default_toggled_style());
            self.common_element_data_mut().style = Style::merge(&self.common_element_data().style, &self.toggled_track_style);
            
            self.pseudo_thumb.common_element_data_mut().style = Style::merge(&self.pseudo_thumb.common_element_data().style, &self.default_toggled_thumb_style());
            self.pseudo_thumb.common_element_data_mut().style = Style::merge(&self.pseudo_thumb.common_element_data().style, &self.toggled_thumb_style);
            *self.common_element_data_mut().style.justify_content_mut() = Some(JustifyContent::FlexEnd);
        };
        
        // Use the toggled styles when state.toggled is true or default_toggled is true.
        if let Some(toggled) = state.toggled {
            if toggled {
                set_toggled_styles();
            }
        } else if default_toggled {
            set_toggled_styles();
        }
        
        let child_node = self.pseudo_thumb.compute_layout(taffy_tree, element_state, scale_factor).unwrap();

        let style: taffy::Style = self.common_element_data.style.to_taffy_style_with_scale_factor(scale_factor);
        self.common_element_data_mut().taffy_node_id = Some(taffy_tree.new_with_children(style, &[child_node]).unwrap());
        self.common_element_data().taffy_node_id
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
        font_system: &mut FontSystem,
    ) {
        let result = taffy_tree.layout(root_node).unwrap();
        self.resolve_layer_rectangle(position, transform, result, z_index);
        self.finalize_borders();

        self.pseudo_thumb.finalize_layout(
            taffy_tree,
            self.pseudo_thumb.common_element_data.taffy_node_id.unwrap(),
            self.common_element_data.computed_layered_rectangle.position,
            z_index,
            transform,
            element_state,
            pointer,
            font_system,
        );
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn on_event(&self, message: OkuMessage, element_state: &mut ElementStateStore, _font_system: &mut FontSystem) -> UpdateResult {
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
            return UpdateResult::default().result_message(OkuMessage::SwitchToggled(state.toggled.unwrap()))
        }

        UpdateResult::default()
    }

    fn initialize_state(&self, _font_system: &mut FontSystem, _scaling_factor: f64) -> ElementStateStoreItem {
        ElementStateStoreItem {
            base: Default::default(),
            data: Box::new(SwitchState::default()),
        }
    }

    fn default_style(&self) -> Style {
        let mut style = Style::default();

        // FIXME: Do not hardcode these sizes.
        *style.display_mut() = Display::Flex;
        *style.align_items_mut() = Some(AlignItems::Center);
        *style.width_mut() = Unit::Px(60.0);
        *style.height_mut() = Unit::Px(34.0);
        *style.padding_mut() = [Unit::Px(4.0), Unit::Px(4.0), Unit::Px(4.0), Unit::Px(4.0)];
        *style.background_mut() = palette::css::LIGHT_GRAY;

        style
    }
}

impl Default for Switch {
    fn default() -> Self {
        Self::new()
    }
}

impl Switch {
    fn default_thumb_style(&self) -> Style {
        let mut style = Style::default();
        
        // FIXME: Do not hardcode these sizes.
        *style.display_mut() = Display::Block;
        *style.width_mut() = Unit::Px(26.0);
        *style.height_mut() = Unit::Px(26.0);
        *style.background_mut() = palette::css::WHITE;
        *style.position_mut() = Position::Relative;
        *style.inset_mut() = [Unit::Px(0.0), Unit::Px(0.0), Unit::Px(0.0), Unit::Px(0.0)];

        style
    }
    
    fn default_toggled_style(&self) -> Style {
        let mut style = Style::default();
        *style.background_mut() = palette::css::DODGER_BLUE;
        Style::merge(&self.default_style(), &style)
    }
    
    fn default_toggled_thumb_style(&self) -> Style {
        let style = Style::default();
        Style::merge(&self.default_thumb_style(), &style)
    }

    pub fn thumb_style(mut self, thumb_style: Style) -> Self {
        self.pseudo_thumb.common_element_data_mut().style = thumb_style;
        self
    }
    
    pub fn toggled_thumb_style(mut self, toggled_thumb_style: Style) -> Self {
        self.toggled_thumb_style = toggled_thumb_style;
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
        element_state.storage.get(&self.common_element_data.component_id).unwrap().data.as_ref().downcast_ref().unwrap()
    }

    pub fn new() -> Switch {
        Switch {
            common_element_data: Default::default(),
            default_toggled: false,
            pseudo_thumb: Container::default(),
            toggled_thumb_style: Default::default(),
            toggled_track_style: Default::default(),
        }
    }
}

impl ElementStyles for Switch {
    fn styles_mut(&mut self) -> &mut Style {
        self.common_element_data.current_style_mut()
    }
}
