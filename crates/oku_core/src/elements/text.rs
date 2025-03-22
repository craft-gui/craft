use crate::components::component::ComponentSpecification;
use crate::components::{Props, UpdateResult};
use crate::elements::cached_editor::CachedEditor;
use crate::elements::common_element_data::CommonElementData;
use crate::elements::element::{Element, ElementBox};
use crate::elements::layout_context::{LayoutContext, TaffyTextContext};
use crate::elements::ElementStyles;
use crate::events::OkuMessage;
use crate::geometry::Point;
use crate::reactive::element_state_store::{ElementStateStore, ElementStateStoreItem};
use crate::style::Style;
use crate::{generate_component_methods_no_children, RendererBox};
use cosmic_text::{Action, Attrs, Buffer, Edit, Family, FontSystem, Weight};
use rustc_hash::FxHasher;
use std::any::Any;
use std::cmp::PartialEq;
use std::hash::Hasher;
use taffy::{NodeId, TaffyTree};

// A stateful element that shows text.
#[derive(Clone, Default, Debug)]
pub struct Text {
    text: String,
    common_element_data: CommonElementData,
}

#[derive(Clone)]
pub struct TextHashValue {
    pub computed_width: f32,
    pub computed_height: f32,
    pub buffer: Buffer,
}

pub struct AttributesRaw {
    pub(crate) font_family_length: u8,
    pub(crate) font_family: Option<[u8; 64]>,
    weight: Weight,
}

impl AttributesRaw {
    pub(crate) fn from(style: &Style) -> Self {
        let font_family = if style.font_family_length() == 0 {
            None
        } else {
            Some(style.font_family_raw())
        };
        Self {
            font_family_length: style.font_family_length(),
            font_family,
            weight: Weight(style.font_weight().0),
        }
    }

    pub(crate) fn to_attrs(&self) -> Attrs {
        let mut attrs = Attrs::new();
        if let Some(font_family) = &self.font_family {
            attrs.family = Family::Name(
                std::str::from_utf8(&font_family[..self.font_family_length as usize]).unwrap()
            );
            attrs.weight = self.weight;
        }
        attrs
    }

}

pub struct TextState<'a> {
    pub cached_editor: CachedEditor<'a>,
    pub dragging: bool,
}

pub(crate) fn hash_text(text: &String) -> u64 {
    let mut text_hasher = FxHasher::default();
    text_hasher.write(text.as_ref());
    text_hasher.finish()
}

impl Text {
    pub fn new(text: &str) -> Text {
        Text {
            text: text.to_string(),
            common_element_data: Default::default(),
        }
    }

    #[allow(dead_code)]
    fn get_state<'a>(&self, element_state: &'a ElementStateStore) -> &'a TextState {
        element_state.storage.get(&self.common_element_data.component_id).unwrap().data.as_ref().downcast_ref().unwrap()
    }
}

impl PartialEq for AttributesRaw {
    fn eq(&self, other: &Self) -> bool {
        self.font_family == other.font_family &&
            self.font_family_length == other.font_family_length &&
            self.weight == other.weight
    }
}

impl Element for Text {
    fn common_element_data(&self) -> &CommonElementData {
        &self.common_element_data
    }

    fn common_element_data_mut(&mut self) -> &mut CommonElementData {
        &mut self.common_element_data
    }

    fn children_mut(&mut self) -> &mut Vec<ElementBox> {
        &mut self.common_element_data.children
    }

    fn name(&self) -> &'static str {
        "Text"
    }

    fn draw(
        &mut self,
        renderer: &mut RendererBox,
        _font_system: &mut FontSystem,
        _taffy_tree: &mut TaffyTree<LayoutContext>,
        _root_node: NodeId,
        _element_state: &ElementStateStore,
        _pointer: Option<Point>,
    ) {
        let computed_layer_rectangle_transformed =
            self.common_element_data.computed_layered_rectangle_transformed;
        let content_rectangle = computed_layer_rectangle_transformed.content_rectangle();

        self.draw_borders(renderer);

        renderer.draw_text(
            self.common_element_data.component_id,
            content_rectangle,
            self.common_element_data.style.color(),
        );
    }

    fn compute_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        _element_state: &mut ElementStateStore,
        scale_factor: f64,
    ) -> Option<NodeId> {
        let style: taffy::Style = self.common_element_data.style.to_taffy_style_with_scale_factor(scale_factor);

        self.common_element_data_mut().taffy_node_id = Some(taffy_tree
            .new_leaf_with_context(
                style,
                LayoutContext::Text(TaffyTextContext::new(self.common_element_data.component_id)),
            )
            .unwrap());

        self.common_element_data().taffy_node_id
    }

    fn finalize_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        root_node: NodeId,
        position: Point,
        z_index: &mut u32,
        transform: glam::Mat4,
        _element_state: &mut ElementStateStore,
        _pointer: Option<Point>,
        _font_system: &mut FontSystem,
    ) {
        let result = taffy_tree.layout(root_node).unwrap();
        self.resolve_layer_rectangle(position, transform, result, z_index);
        
        self.finalize_borders();
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn initialize_state(&self, font_system: &mut FontSystem, scaling_factor: f64) -> ElementStateStoreItem {
        let cached_editor = CachedEditor::new(&self.text, &self.common_element_data.style, scaling_factor, font_system);
        let text_state = TextState {
            cached_editor,
            dragging: false,
        };

        ElementStateStoreItem {
            base: Default::default(),
            data: Box::new(text_state)
        }
    }

    fn update_state(&self, font_system: &mut FontSystem, element_state: &mut ElementStateStore, reload_fonts: bool, scaling_factor: f64) {
        let state: &mut TextState = element_state
            .storage
            .get_mut(&self.common_element_data.component_id)
            .unwrap()
            .data
            .as_mut()
            .downcast_mut()
            .unwrap();
        
        state.cached_editor.update_state(&self.text, &self.common_element_data.style, scaling_factor, reload_fonts, font_system);
    }

    fn on_event(
        &self,
        message: OkuMessage,
        element_state: &mut ElementStateStore,
        font_system: &mut FontSystem,
    ) -> UpdateResult {
        let state: &mut TextState = element_state
            .storage
            .get_mut(&self.common_element_data.component_id)
            .unwrap()
            .data
            .as_mut()
            .downcast_mut()
            .unwrap();

        let cached_editor = &mut state.cached_editor;
        let content_rect = self.common_element_data.computed_layered_rectangle.content_rectangle();
        let content_position = content_rect.position();

        // Handle selection.
        match message {
            OkuMessage::PointerButtonEvent(pointer_button) => {
                let pointer_position = pointer_button.position;
                let pointer_content_position = pointer_position - content_position;
                if pointer_button.state.is_pressed() && content_rect.contains(&pointer_button.position) {
                    cached_editor.editor.action(
                        font_system,
                        Action::Click {
                            x: pointer_content_position.x as i32,
                            y: pointer_content_position.y as i32,
                        },
                    );
                    state.dragging = true;
                } else {
                    state.dragging = false;
                }
                UpdateResult::new().prevent_defaults().prevent_propagate()
            }
            OkuMessage::PointerMovedEvent(moved) => {
                if state.dragging {
                    let pointer_position = moved.position;
                    let pointer_content_position = pointer_position - content_position;
                    cached_editor.editor.action(
                        font_system,
                        Action::Drag {
                            x: pointer_content_position.x as i32,
                            y: pointer_content_position.y as i32,
                        },
                    );
                }
                UpdateResult::new().prevent_defaults().prevent_propagate()
            }
            _ => UpdateResult::new(),
        }
    }
}

impl Text {

    generate_component_methods_no_children!();
}

impl ElementStyles for Text {
    fn styles_mut(&mut self) -> &mut Style {
        self.common_element_data.current_style_mut()
    }
}
