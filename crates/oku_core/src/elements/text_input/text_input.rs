use crate::components::component::{ComponentId, ComponentSpecification};
use crate::elements::element::{Element, ElementBox};
use crate::elements::layout_context::{AvailableSpace, LayoutContext, TaffyTextInputContext};
use crate::elements::ElementStyles;
use crate::reactive::element_state_store::{ElementStateStore, ElementStateStoreItem};
use crate::style::{Style, Unit};
use crate::{generate_component_methods_private_push, RendererBox};
use parley::FontContext;
use peniko::Brush;
use rustc_hash::FxHasher;
use std::any::Any;
use std::collections::HashMap;
use std::hash::Hasher;
use taffy::{NodeId, TaffyTree};

use crate::components::Props;
use crate::components::UpdateResult;
use crate::elements::common_element_data::CommonElementData;
use crate::elements::text::parley::{TextHashKey, TextHashValue};
use crate::elements::text_input::editor::Editor;
use crate::events::OkuMessage;
use crate::geometry::Point;
use crate::Color;

// A stateful element that shows a text input.
#[derive(Clone, Default, Debug)]
pub struct TextInput {
    text: String,
    common_element_data: CommonElementData,
}

pub struct TextInputState {
    pub id: ComponentId,
    pub text: String,
    pub cached_text_layout: HashMap<TextHashKey, TextHashValue>,
    pub last_cache_key: Option<TextHashKey>,
    pub editor: Editor,
    pub children: Vec<ComponentSpecification>,
    pub style: Style,
    /// We need to update the text layout in finalize_layout if the constraints or available space have changed since the last layout pass
    /// AND the last layout operation was a text size cache hit.
    ///
    /// This may be true because the cached text size that we retrieve does not map to the current layout which is computed during the last cache miss.
    pub should_recompute_final_text_layout: bool,
    pub reload_fonts: bool,
    pub text_hash: u64,
}

impl TextInputState {
    pub(crate) fn new(id: ComponentId, text: &str, style: Style) -> Self {
        Self {
            id,
            text: text.to_string(),
            cached_text_layout: Default::default(),
            last_cache_key: None,
            editor: Editor::new(text, style),
            children: Vec::new(),
            style: Default::default(),
            should_recompute_final_text_layout: false,
            reload_fonts: false,
            text_hash: 0,
        }
    }
}

impl TextInputState {
    /// Measure the width and height of the text given layout constraints.
    pub(crate) fn measure(
        &mut self,
        known_dimensions: taffy::Size<Option<f32>>,
        available_space: taffy::Size<taffy::AvailableSpace>,
        font_context: &mut FontContext,
        font_layout_context: &mut parley::LayoutContext<Brush>,
        text_hash: u64,
        font_settings_hash: u64,
    ) -> taffy::Size<f32> {
        // Set width constraint
        let width_constraint = known_dimensions.width.or(match available_space.width {
            taffy::AvailableSpace::MinContent => Some(0.0),
            taffy::AvailableSpace::MaxContent => None,
            taffy::AvailableSpace::Definite(width) => Some(width),
        });

        let height_constraint = known_dimensions.height;

        let available_space_width_u32: AvailableSpace = match available_space.width {
            taffy::AvailableSpace::MinContent => AvailableSpace::MinContent,
            taffy::AvailableSpace::MaxContent => AvailableSpace::MaxContent,
            taffy::AvailableSpace::Definite(width) => AvailableSpace::Definite(width.to_bits()),
        };
        let available_space_height_u32: AvailableSpace = match available_space.height {
            taffy::AvailableSpace::MinContent => AvailableSpace::MinContent,
            taffy::AvailableSpace::MaxContent => AvailableSpace::MaxContent,
            taffy::AvailableSpace::Definite(height) => AvailableSpace::Definite(height.to_bits()),
        };

        let cache_key = TextHashKey {
            text_hash,
            font_settings_hash,
            width_constraint: width_constraint.map(|w| w.to_bits()),
            height_constraint: height_constraint.map(|h| h.to_bits()),
            available_space_width: available_space_width_u32,
            available_space_height: available_space_height_u32,
        };

        // If the text or font settings have changed since the last cache, we have to recompute the size of our text.
        let mut text_changed = true;
        if let Some(last_cache_key) = &self.last_cache_key {
            if last_cache_key.text_hash == cache_key.text_hash
                && last_cache_key.font_settings_hash == cache_key.font_settings_hash
            {
                text_changed = false;
            }
        }

        let previous_cache_key = self.last_cache_key.clone();
        // Update the current cache key.
        self.last_cache_key = Some(cache_key.clone());

        // Use the cached size if possible and if the text/font settings haven't changed.
        if !self.reload_fonts && self.cached_text_layout.contains_key(&cache_key) && !text_changed {
            let computed_size = self.cached_text_layout.get(&cache_key).unwrap();

            let previous_cache_key = previous_cache_key.unwrap();
            let same_available_space = previous_cache_key.available_space_width == cache_key.available_space_width
                && previous_cache_key.available_space_height == cache_key.available_space_height;
            let same_constraints = previous_cache_key.width_constraint == cache_key.width_constraint
                && previous_cache_key.height_constraint == cache_key.height_constraint;

            // The layout gets updated for each new cache entry, so we may need to recompute the final text layout in Text::finalize_layout.
            // We need to recompute the final layout if the constraints or available space have changed since the last layout pass.
            if !same_constraints || !same_available_space {
                self.should_recompute_final_text_layout = true;
            }

            taffy::Size {
                width: computed_size.computed_width,
                height: computed_size.computed_height,
            }
        } else {
            // Cache is not available or the text/font settings have changed, so we need to recompute the size.
            self.editor.editor.set_width(width_constraint);
            self.editor.editor.update_layout(font_context, font_layout_context);
            let width = self.editor.editor.layout.width();
            let height = self.editor.editor.layout.height();

            let computed_size = TextHashValue {
                computed_width: width,
                computed_height: height,
            };
            
            // Update the cache.
            self.cached_text_layout.insert(cache_key.clone(), computed_size);
            self.should_recompute_final_text_layout = false;

            taffy::Size {
                width: computed_size.computed_width,
                height: computed_size.computed_height,
            }
        }
    }
}

impl TextInput {
    pub fn new(text: &str) -> TextInput {
        TextInput {
            text: String::from(text),
            common_element_data: Default::default(),
        }
    }

    #[allow(dead_code)]
    fn get_state<'a>(&self, element_state: &'a ElementStateStore) -> &'a TextInputState {
        element_state.storage.get(&self.common_element_data.component_id).unwrap().data.as_ref().downcast_ref().unwrap()
    }

    fn get_state_mut<'a>(&self, element_state: &'a mut ElementStateStore) -> &'a mut TextInputState {
        element_state
            .storage
            .get_mut(&self.common_element_data.component_id)
            .unwrap()
            .data
            .as_mut()
            .downcast_mut()
            .unwrap()
    }
}

impl Element for TextInput {
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
        "Text Input"
    }

    fn draw(
        &mut self,
        renderer: &mut RendererBox,
        _font_context: &mut FontContext,
        _taffy_tree: &mut TaffyTree<LayoutContext>,
        _root_node: NodeId,
        _element_state: &ElementStateStore,
        _pointer: Option<Point>,
    ) {
        let computed_layer_rectangle_transformed = self.common_element_data.computed_layered_rectangle_transformed;
        let content_rectangle = computed_layer_rectangle_transformed.content_rectangle();

        self.draw_borders(renderer);

        renderer.draw_text(self.common_element_data.component_id, content_rectangle);
    }

    fn compute_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        element_state: &mut ElementStateStore,
        scale_factor: f64,
    ) -> Option<NodeId> {
        self.merge_default_style();
        let style: taffy::Style = self.common_element_data.style.to_taffy_style_with_scale_factor(scale_factor);

        let state = self.get_state_mut(element_state);
        let mut font_settings_hasher = FxHasher::default();
        let mut text_hasher = FxHasher::default();
        let mut hash_font_settings = |style: &Style| {
            font_settings_hasher.write_u8(style.font_family_length());
            font_settings_hasher.write(&style.font_family_raw());
            font_settings_hasher.write_u32(style.font_size().to_bits());
            font_settings_hasher.write_u16(style.font_weight().0);
            font_settings_hasher.write_usize(style.font_style() as usize);
        };
        text_hasher.write(state.editor.editor.buffer.as_bytes());
        hash_font_settings(&self.common_element_data.style);

        let text_hash = text_hasher.finish();
        let font_settings_hash = font_settings_hasher.finish();
        
        self.common_element_data_mut().taffy_node_id = Some(
            taffy_tree
                .new_leaf_with_context(
                    style,
                    LayoutContext::TextInput(TaffyTextInputContext::new(self.common_element_data.component_id, text_hash, font_settings_hash)),
                )
                .unwrap(),
        );

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
        _pointer: Option<Point>,
        font_context: &mut FontContext,
        layout_context: &mut parley::LayoutContext<Brush>,
    ) {
        let state = self.get_state_mut(element_state);
        if state.should_recompute_final_text_layout {
            if let Some(last_cache_key) = &state.last_cache_key {
                state.editor.editor.set_width(last_cache_key.width_constraint.map(f32::from_bits));
                state.editor.editor.update_layout(font_context, layout_context);
            }
        }
        
        let result = taffy_tree.layout(root_node).unwrap();
        self.resolve_layer_rectangle(position, transform, result, z_index);
        self.finalize_borders();
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn on_event(&self, message: OkuMessage, element_state: &mut ElementStateStore) -> UpdateResult {
        let state = self.get_state_mut(element_state);

        let text_position = self.common_element_data().computed_layered_rectangle_transformed.content_rectangle();
        let update_result = state.editor.handle_event(message, text_position.x, text_position.y);

        update_result.unwrap_or_default()
    }

    fn initialize_state(&self) -> ElementStateStoreItem {
        let mut state = TextInputState::new(self.common_element_data.component_id, &self.text, *self.style());

        self.update_state_fragments(&mut state);
        
        ElementStateStoreItem {
            base: Default::default(),
            data: Box::new(state),
        }
    }

    fn update_state(&self, element_state: &mut ElementStateStore, _reload_fonts: bool) {
        let state = self.get_state_mut(element_state);
        self.update_state_fragments(state);

        // FIXME: We will need to rewrite this when we do text input caching, but for now we still need this here, so that we can detect default text changes from the user.
        let mut hasher = FxHasher::default();
        hasher.write(state.text.as_bytes());
        let text_hash = hasher.finish();

        if text_hash != state.text_hash {
            state.editor.editor.set_text(&state.text);
        }

        state.text_hash = text_hash;
        
    }

    fn default_style(&self) -> Style {
        let mut style= Style::default();

        const BORDER_COLOR: Color = Color::from_rgb8(199, 199, 206);
        *style.border_color_mut() = [BORDER_COLOR; 4];
        *style.border_width_mut() = [Unit::Px(1.0); 4];
        *style.border_radius_mut() = [(5.0, 5.0); 4];
        let vertical_padding = Unit::Px(2.0);
        let horizontal_padding = Unit::Px(8.0);
        *style.padding_mut() = [vertical_padding, horizontal_padding, vertical_padding, horizontal_padding];
        
        style
    }
}

impl TextInput {
    generate_component_methods_private_push!();

    fn update_state_fragments(&self, state: &mut TextInputState) {
        state.id = self.common_element_data.component_id;
        state.text = self.text.clone();
        state.children = self.common_element_data.child_specs.clone();
        state.style = *self.style();
    }
}

impl ElementStyles for TextInput {
    fn styles_mut(&mut self) -> &mut Style {
        self.common_element_data.current_style_mut()
    }
}
