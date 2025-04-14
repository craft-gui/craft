use crate::components::component::ComponentSpecification;
use crate::components::Props;
use crate::components::UpdateResult;
use crate::elements::element::{Element, ElementBoxed};
use crate::elements::element_data::ElementData;
use crate::elements::layout_context::{LayoutContext, TaffyTextInputContext};
use crate::elements::scroll_state::ScrollState;
use crate::elements::ElementStyles;
use crate::events::CraftMessage;
use crate::geometry::{Point, Size};
use crate::reactive::element_state_store::{ElementStateStore, ElementStateStoreItem};
use crate::renderer::color::Color;
use crate::renderer::renderer::TextScroll;
use crate::style::{Display, Style, Unit};
use crate::{generate_component_methods_no_children, RendererBox};
use std::any::Any;
use std::sync::Arc;
use taffy::{NodeId, TaffyTree};
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::Ime;
use winit::keyboard::{Key, NamedKey};
use winit::window::Window;

// A stateful element that shows text.
#[derive(Clone, Default, Debug)]
pub struct TextInput {
    text: String,
    element_data: ElementData,
    /// Whether the text input will update the editor every update with the user provided text.
    /// NOTE: The editor will always use the user provided text on initialization.
    use_text_value_on_update: bool,
}

#[derive(Clone, Default, Debug)]
pub(crate) struct ImeState {
    pub is_ime_active: bool,
}

pub struct TextInputState {
    pub is_active: bool,
    pub(crate) scroll_state: ScrollState,
    pub(crate) ime_state: ImeState,
}

impl TextInput {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
            element_data: ElementData::default(),
            use_text_value_on_update: true,
        }
    }

    #[allow(dead_code)]
    fn get_state<'a>(&self, element_state: &'a ElementStateStore) -> &'a TextInputState {
        element_state.storage.get(&self.element_data.component_id).unwrap().data.as_ref().downcast_ref().unwrap()
    }
}

impl Element for TextInput {
    fn element_data(&self) -> &ElementData {
        &self.element_data
    }

    fn element_data_mut(&mut self) -> &mut ElementData {
        &mut self.element_data
    }

    fn children_mut(&mut self) -> &mut Vec<ElementBoxed> {
        &mut self.element_data.children
    }

    fn name(&self) -> &'static str {
        "TextInput"
    }

    fn draw(
        &mut self,
        renderer: &mut RendererBox,
        _taffy_tree: &mut TaffyTree<LayoutContext>,
        _root_node: NodeId,
        element_state: &mut ElementStateStore,
        _pointer: Option<Point>,
        window: Option<Arc<dyn Window>>,
    ) {
        if !self.element_data.style.visible() {
            return;
        }
        let computed_box_transformed = self.element_data.computed_box_transformed;
        let content_rectangle = computed_box_transformed.content_rectangle();

        self.draw_borders(renderer);

        let is_scrollable = self.element_data.is_scrollable();

        if is_scrollable {
            self.maybe_start_layer(renderer);
        }

        let scroll_y = if let Some(state) =
            element_state.storage.get(&self.element_data.component_id).unwrap().data.downcast_ref::<TextInputState>()
        {
            state.scroll_state.scroll_y
        } else {
            0.0
        };

        let text_scroll = if is_scrollable {
            Some(TextScroll::new(scroll_y, self.element_data.computed_scroll_track.height))
        } else {
            None
        };

        if let Some(state) =
            element_state.storage.get_mut(&self.element_data.component_id).unwrap().data.downcast_mut::<TextInputState>()
        {
            let fill_color = self.element_data.style.color();
            
            renderer.draw_text(
                content_rectangle,
                text_scroll,
                true
            );
        }

        if let Some(state) = element_state
            .storage
            .get_mut(&self.element_data.component_id)
            .unwrap()
            .data
            .downcast_mut::<TextInputState>()
        {
            state.is_active = false;
        }

        if is_scrollable {
            self.maybe_end_layer(renderer);
        }

        self.draw_scrollbar(renderer);
    }

    fn compute_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        _element_state: &mut ElementStateStore,
        scale_factor: f64,
    ) -> Option<NodeId> {
        self.merge_default_style();
        let style: taffy::Style = self.element_data.style.to_taffy_style_with_scale_factor(scale_factor);

        self.element_data_mut().taffy_node_id = Some(
            taffy_tree
                .new_leaf_with_context(
                    style,
                    LayoutContext::TextInput(TaffyTextInputContext::new(self.element_data.component_id)),
                )
                .unwrap(),
        );

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
        _pointer: Option<Point>,
    ) {
        let result = taffy_tree.layout(root_node).unwrap();
        self.resolve_box(position, transform, result, z_index);
        self.finalize_borders();

        self.element_data.scrollbar_size = Size::new(result.scrollbar_size.width, result.scrollbar_size.height);
        self.element_data.computed_scrollbar_size = Size::new(result.scroll_width(), result.scroll_height());

        let scroll_y = if let Some(container_state) =
            element_state.storage.get(&self.element_data.component_id).unwrap().data.downcast_ref::<TextInputState>()
        {
            container_state.scroll_state.scroll_y
        } else {
            0.0
        };

        self.finalize_scrollbar(scroll_y);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn on_event(
        &self,
        message: &CraftMessage,
        element_state: &mut ElementStateStore,
    ) -> UpdateResult {
        UpdateResult::new()
    }

    fn initialize_state(&self, scaling_factor: f64) -> ElementStateStoreItem {
        let text_input_state = TextInputState {
            ime_state: ImeState::default(),
            is_active: false,
            scroll_state: ScrollState::default(),
        };

        ElementStateStoreItem {
            base: Default::default(),
            data: Box::new(text_input_state),
        }
    }

    fn update_state(
        &self,
        element_state: &mut ElementStateStore,
        reload_fonts: bool,
        scaling_factor: f64,
    ) {
        let state: &mut TextInputState = element_state
            .storage
            .get_mut(&self.element_data.component_id)
            .unwrap()
            .data
            .as_mut()
            .downcast_mut()
            .unwrap();
    }

    fn default_style(&self) -> Style {
        let mut style = Style::default();
        *style.display_mut() = Display::Block;
        const BORDER_COLOR: Color = Color::from_rgb8(199, 199, 206);
        *style.border_color_mut() = [BORDER_COLOR; 4];
        *style.border_width_mut() = [Unit::Px(1.0); 4];
        *style.border_radius_mut() = [(5.0, 5.0); 4];
        let padding = Unit::Px(4.0);
        *style.padding_mut() = [padding, padding, padding, padding];

        style
    }
}

impl TextInput {
    generate_component_methods_no_children!();

    /// Whether the text input will update the editor every update with the user provided text.
    /// NOTE: The editor will always use the user provided text on initialization.
    pub fn use_text_value_on_update(mut self, use_initial_text_value: bool) -> Self {
        self.use_text_value_on_update = use_initial_text_value;
        self
    }
}

impl ElementStyles for TextInput {
    fn styles_mut(&mut self) -> &mut Style {
        self.element_data.current_style_mut()
    }
}
