use crate::components::component::{ComponentOrElement, ComponentSpecification};
use crate::components::UpdateResult;
use crate::elements::element_data::ElementData;
use crate::elements::element_states::ElementState;
use crate::elements::layout_context::LayoutContext;
use crate::events::OkuMessage;
use crate::geometry::borders::BorderSpec;
use crate::geometry::side::Side;
use crate::geometry::{Border, ElementRectangle, Margin, Padding, Point, Rectangle, Size};
use crate::reactive::element_state_store::{ElementStateStore, ElementStateStoreItem};
use crate::style::Style;
use crate::RendererBox;
use std::any::Any;
use std::fmt::Debug;
use std::sync::Arc;
use cosmic_text::FontSystem;
use taffy::{NodeId, Overflow, Position, TaffyTree};
use winit::window::Window;

#[derive(Clone, Debug)]
pub struct ElementBox {
    pub(crate) internal: Box<dyn Element>,
}

pub(crate) trait Element: Any + StandardElementClone + Debug + Send + Sync {
    fn element_data(&self) -> &ElementData;
    fn element_data_mut(&mut self) -> &mut ElementData;

    fn children(&self) -> Vec<&dyn Element> {
        self.element_data().children.iter().map(|x| x.internal.as_ref()).collect()
    }

    fn children_mut(&mut self) -> &mut Vec<ElementBox> {
        &mut self.element_data_mut().children
    }

    fn style(&self) -> &Style {
        &self.element_data().style
    }

    fn style_mut(&mut self) -> &mut Style {
        &mut self.element_data_mut().style
    }

    fn in_bounds(&self, point: Point) -> bool {
        let element_data = self.element_data();

        let transformed_border_rectangle =
            element_data.computed_box_transformed.border_rectangle();

        transformed_border_rectangle.contains(&point)
    }

    fn get_id(&self) -> &Option<String> {
        &self.element_data().id
    }

    #[allow(dead_code)]
    fn id(&mut self, id: Option<&str>) -> Box<dyn Element> {
        self.element_data_mut().id = id.map(String::from);
        self.clone_box()
    }

    fn component_id(&self) -> u64 {
        self.element_data().component_id
    }

    fn taffy_node_id(&self) -> Option<NodeId> {
        self.element_data().taffy_node_id
    }

    fn set_component_id(&mut self, id: u64) {
        self.element_data_mut().component_id = id;
    }

    fn name(&self) -> &'static str;

    #[allow(clippy::too_many_arguments)]
    fn draw(
        &mut self,
        renderer: &mut RendererBox,
        font_system: &mut FontSystem,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        root_node: NodeId,
        element_state: &mut ElementStateStore,
        pointer: Option<Point>,
        window: Option<Arc<dyn Window>>
    );

    fn compute_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        element_state: &mut ElementStateStore,
        scale_factor: f64,
    ) -> Option<NodeId>;

    /// Finalizes the layout of the element.
    ///
    /// The majority of the layout computation is done in the `compute_layout` method.
    /// Store the computed values in the `element_data` struct.
    #[allow(clippy::too_many_arguments)]
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
    );

    fn as_any(&self) -> &dyn Any;

    fn on_event(&self, _message: OkuMessage, _element_state: &mut ElementStateStore, _font_system: &mut FontSystem) -> UpdateResult {
        UpdateResult::default()
    }

    fn resolve_box(
        &mut self,
        relative_position: Point,
        scroll_transform: glam::Mat4,
        result: &taffy::Layout,
        layout_order: &mut u32,
    ) {
        let element_data_mut = self.element_data_mut();
        element_data_mut.layout_order = *layout_order;
        *layout_order += 1;

        let position = match element_data_mut.style.position() {
            Position::Relative => relative_position + result.location.into(),
            // We'll need to create our own enum for this because currently, relative acts more like static and absolute acts like relative.
            Position::Absolute => relative_position + result.location.into(),
        };

        let mut size = result.size.into();
        // FIXME: Don't use the content size for position absolute containers.
        // The following is a broken layout using result.size.
        // └──  FLEX COL [x: 1    y: 44   w: 140  h: 45   content_w: 139  content_h: 142  border: l:1 r:1 t:1 b:1, padding: l:12 r:12 t:8 b:8] (NodeId(4294967303))
        //     ├──  LEAF [x: 13   y: 9    w: 114  h: 25   content_w: 29   content_h: 25   border: l:0 r:0 t:0 b:0, padding: l:0 r:0 t:0 b:0] (NodeId(4294967298))
        //     ├──  LEAF [x: 13   y: 34   w: 114  h: 25   content_w: 29   content_h: 25   border: l:0 r:0 t:0 b:0, padding: l:0 r:0 t:0 b:0] (NodeId(4294967299))
        //     ├──  LEAF [x: 13   y: 59   w: 114  h: 25   content_w: 29   content_h: 25   border: l:0 r:0 t:0 b:0, padding: l:0 r:0 t:0 b:0] (NodeId(4294967300))
        //     ├──  LEAF [x: 13   y: 84   w: 114  h: 25   content_w: 29   content_h: 25   border: l:0 r:0 t:0 b:0, padding: l:0 r:0 t:0 b:0] (NodeId(4294967301))
        //     └──  LEAF [x: 13   y: 109  w: 114  h: 25   content_w: 29   content_h: 25   border: l:0 r:0 t:0 b:0, padding: l:0 r:0 t:0 b:0] (NodeId(4294967302))
        if element_data_mut.style.position() == Position::Absolute {
            size = Size::new(f32::max(result.size.width, result.content_size.width), f32::max(result.size.height, result.content_size.height));
        }
        
        element_data_mut.content_size =
            Size::new(result.content_size.width, result.content_size.height);
        element_data_mut.computed_box = ElementRectangle {
            margin: Margin::new(result.margin.top, result.margin.right, result.margin.bottom, result.margin.left),
            border: Border::new(result.border.top, result.border.right, result.border.bottom, result.border.left),
            padding: Padding::new(result.padding.top, result.padding.right, result.padding.bottom, result.padding.left),
            position,
            size,
        };
        element_data_mut.computed_box_transformed =
            element_data_mut.computed_box.transform(scroll_transform);
    }

    fn draw_children(
        &mut self,
        renderer: &mut RendererBox,
        font_system: &mut FontSystem,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        element_state: &mut ElementStateStore,
        pointer: Option<Point>,
        window: Option<Arc<dyn Window>>
    ) {
        for child in self.element_data_mut().children.iter_mut() {
            let taffy_child_node_id = child.internal.taffy_node_id();
            // Skip non-visual elements.
            if taffy_child_node_id.is_none() {
                continue;
            }
            child.internal.draw(
                renderer,
                font_system,
                taffy_tree,
                taffy_child_node_id.unwrap(),
                element_state,
                pointer,
                window.clone(),
            );
        }
    }

    fn draw_borders(&self, renderer: &mut RendererBox) {
        let element_data = self.element_data();
        let current_style = element_data.current_style();
        let background_color = current_style.background();

        // OPTIMIZATION: Draw a normal rectangle if no border values have been modified.
        if !current_style.has_border() {
            renderer.draw_rect(
                element_data.computed_box_transformed.padding_rectangle(),
                background_color,
            );
            return;
        }

        let computed_border_spec = &element_data.computed_border;

        let background_path = computed_border_spec.build_background_path();
        renderer.fill_bez_path(background_path, background_color);

        let top = computed_border_spec.get_side(Side::Top);
        let right = computed_border_spec.get_side(Side::Right);
        let bottom = computed_border_spec.get_side(Side::Bottom);
        let left = computed_border_spec.get_side(Side::Left);

        let border_top_path = computed_border_spec.build_side_path(Side::Top);
        let border_right_path = computed_border_spec.build_side_path(Side::Right);
        let border_bottom_path = computed_border_spec.build_side_path(Side::Bottom);
        let border_left_path = computed_border_spec.build_side_path(Side::Left);

        renderer.fill_bez_path(border_top_path, top.color);
        renderer.fill_bez_path(border_right_path, right.color);
        renderer.fill_bez_path(border_bottom_path, bottom.color);
        renderer.fill_bez_path(border_left_path, left.color);
    }

    fn should_start_new_layer(&self) -> bool {
        let element_data = self.element_data();

        element_data.current_style().overflow()[1] == Overflow::Scroll
    }

    fn maybe_start_layer(&self, renderer: &mut RendererBox) {
        let element_data = self.element_data();
        let padding_rectangle = element_data.computed_box_transformed.padding_rectangle();

        if self.should_start_new_layer() {
            renderer.push_layer(padding_rectangle);
        }
    }

    fn maybe_end_layer(&self, renderer: &mut RendererBox) {
        if self.should_start_new_layer() {
            renderer.pop_layer();
        }
    }

    fn finalize_borders(&mut self) {
        let element_data = self.element_data_mut();

        // OPTIMIZATION: Don't compute the border if no border style values have been modified.
        if !element_data.current_style().has_border() {
            return;
        }

        let element_rect = element_data.computed_box_transformed;
        let borders = element_rect.border;
        let border_spec = BorderSpec::new(
            element_rect.border_rectangle(),
            [borders.top, borders.right, borders.bottom, borders.left],
            element_data.current_style().border_radius(),
            element_data.current_style().border_color(),
        );
        element_data.computed_border = border_spec.compute_border_spec();
    }

    fn draw_scrollbar(&mut self, renderer: &mut RendererBox) {
        let scrollbar_color = self.element_data().current_style().scrollbar_color();

        // track
        renderer.draw_rect(self.element_data_mut().computed_scroll_track, scrollbar_color.track_color);

        // thumb
        renderer.draw_rect(self.element_data_mut().computed_scroll_thumb, scrollbar_color.thumb_color);
    }

    fn finalize_scrollbar(&mut self, scroll_y: f32) {
        let element_data = self.element_data_mut();
        if element_data.style.overflow()[1] != Overflow::Scroll {
            return;
        }
        let box_transformed = element_data.computed_box_transformed;

        // Client Height = padding box height.
        let client_height = box_transformed.padding_rectangle().height;

        // Taffy is not adding the padding bottom to the content height, so we'll add it here.
        // Content Size = overflowed content size + padding
        // Scroll Height = Content Size
        let scroll_height = element_data.content_size.height + box_transformed.padding.bottom;
        let scroll_track_width = element_data.scrollbar_size.width;

        // The scroll track height is the height of the padding box.
        let scroll_track_height = client_height;

        let max_scroll_y = (scroll_height - client_height).max(0.0);
        element_data.max_scroll_y = max_scroll_y;

        let visible_y = client_height / scroll_height;
        let scroll_thumb_height = scroll_track_height * visible_y;
        let remaining_height = scroll_track_height - scroll_thumb_height;
        let scroll_thumb_offset = if max_scroll_y != 0.0 { scroll_y / max_scroll_y * remaining_height } else { 0.0 };

        element_data.computed_scroll_track = Rectangle::new(
            box_transformed.position.x + box_transformed.size.width
                - scroll_track_width
                - box_transformed.border.right,
            box_transformed.position.y + box_transformed.border.top,
            scroll_track_width,
            scroll_track_height,
        );

        let scroll_thumb_width = scroll_track_width;
        element_data.computed_scroll_thumb = element_data.computed_scroll_track;
        element_data.computed_scroll_thumb.y += scroll_thumb_offset;
        element_data.computed_scroll_thumb.width = scroll_thumb_width;
        element_data.computed_scroll_thumb.height = scroll_thumb_height;
    }

    /// Called when the element is assigned a unique component id.
    fn initialize_state(&self, _font_system: &mut FontSystem, _scaling_factor: f64) -> ElementStateStoreItem {
        ElementStateStoreItem {
            base: Default::default(),
            data: Box::new(()),
        }
    }

    #[allow(dead_code)]
    fn finalize_state(&mut self, element_state: &mut ElementStateStore, pointer: Option<Point>) {
        let element_data = self.element_data_mut();
        let element_state = element_state.storage.get_mut(&element_data.component_id).unwrap();
        element_state.base.current_state = ElementState::Normal;

        let border_rectangle = element_data.computed_box_transformed.border_rectangle();

        if let Some(pointer) = pointer {
            if border_rectangle.contains(&pointer) {
                element_state.base.current_state = ElementState::Hovered;
            }
        }
    }

    fn get_base_state<'a>(&self, element_state: &'a ElementStateStore) -> &'a ElementStateStoreItem {
        element_state.storage.get(&self.element_data().component_id).unwrap()
    }

    #[allow(dead_code)]
    fn get_base_state_mut<'a>(&self, element_state: &'a mut ElementStateStore) -> &'a mut ElementStateStoreItem {
        element_state.storage.get_mut(&self.element_data().component_id).unwrap()
    }

    /// Called on sequential renders to update any state that the element may have.
    fn update_state(&self, _font_system: &mut FontSystem, _element_state: &mut ElementStateStore, _reload_fonts: bool, _scaling_factor: f64) {}

    fn default_style(&self) -> Style {
        Style::default()
    }

    fn merge_default_style(&mut self) {
        self.element_data_mut().style = Style::merge(&self.default_style(), &self.element_data().style);
    }
}

impl<T: Element> From<T> for ElementBox {
    fn from(element: T) -> Self {
        ElementBox {
            internal: Box::new(element),
        }
    }
}

impl<T: Element> From<T> for ComponentOrElement {
    fn from(element: T) -> Self {
        ComponentOrElement::Element(element.into())
    }
}

impl From<ElementBox> for ComponentOrElement {
    fn from(element: ElementBox) -> Self {
        ComponentOrElement::Element(element)
    }
}

impl From<ElementBox> for ComponentSpecification {
    fn from(element: ElementBox) -> Self {
        let key = element.internal.element_data().key.clone();
        let children = element.internal.element_data().child_specs.clone();
        let props = element.internal.element_data().props.clone();
        ComponentSpecification {
            component: ComponentOrElement::Element(element),
            key,
            props,
            children,
        }
    }
}

impl<T> From<T> for ComponentSpecification
where
    T: Element,
{
    fn from(element: T) -> Self {
        let key = element.element_data().key.clone();
        let children_specs = element.element_data().child_specs.clone();
        let props = element.element_data().props.clone();
        ComponentSpecification {
            component: ComponentOrElement::Element(element.into()),
            key,
            props,
            children: children_specs,
        }
    }
}

impl dyn Element {
    #[allow(dead_code)]
    pub fn print_tree(&self) {
        let mut elements: Vec<(&dyn Element, usize, bool)> = vec![(self, 0, true)];
        while let Some((element, indent, is_last)) = elements.pop() {
            let mut prefix = String::new();
            for _ in 0..indent {
                prefix.push_str("  ");
            }
            if is_last {
                prefix.push_str("└─");
            } else {
                prefix.push_str("├─");
            }
            println!(
                "{}{}, Component Id: {} Id: {:?}",
                prefix,
                element.name(),
                element.component_id(),
                element.get_id()
            );
            let children = element.children();
            for (i, child) in children.iter().enumerate().rev() {
                let is_last = i == children.len() - 1;
                elements.push((*child, indent + 1, is_last));
            }
        }
    }
}

pub trait StandardElementClone {
    fn clone_box(&self) -> Box<dyn Element>;
}

impl<T> StandardElementClone for T
where
    T: Element + Clone,
{
    fn clone_box(&self) -> Box<dyn Element> {
        Box::new(self.clone())
    }
}

// We can now implement Clone manually by forwarding to clone_box.
impl Clone for Box<dyn Element> {
    fn clone(&self) -> Box<dyn Element> {
        self.clone_box()
    }
}

#[macro_export]
macro_rules! generate_component_methods_no_children {
    () => {
        #[allow(dead_code)]
        pub fn component(self) -> ComponentSpecification {
            ComponentSpecification::new(self.into())
        }

        #[allow(dead_code)]
        pub fn key(mut self, key: &str) -> Self {
            self.element_data.key = Some(key.to_string());

            self
        }

        #[allow(dead_code)]
        pub fn props(mut self, props: Props) -> Self {
            self.element_data.props = Some(props);

            self
        }

        #[allow(dead_code)]
        pub fn id(mut self, id: &str) -> Self {
            self.element_data.id = Some(id.to_string());
            self
        }

        #[allow(dead_code)]
        pub fn hovered(mut self) -> Self {
            self.element_data.current_state = $crate::elements::element_states::ElementState::Hovered;
            self
        }

        #[allow(dead_code)]
        pub fn pressed(mut self) -> Self {
            self.element_data.current_state = $crate::elements::element_states::ElementState::Pressed;
            self
        }

        #[allow(dead_code)]
        pub fn disabled(mut self) -> Self {
            self.element_data.current_state = $crate::elements::element_states::ElementState::Disabled;
            self
        }

        #[allow(dead_code)]
        pub fn focused(mut self) -> Self {
            self.element_data.current_state = $crate::elements::element_states::ElementState::Focused;
            self
        }
    };
}

#[macro_export]
macro_rules! generate_component_methods_private_push {
    () => {
        $crate::generate_component_methods_no_children!();

        #[allow(dead_code)]
        fn push<T>(mut self, component_specification: T) -> Self
        where
            T: Into<ComponentSpecification>,
        {
            self.element_data.child_specs.push(component_specification.into());

            self
        }

        #[allow(dead_code)]
        fn push_children<T>(mut self, children: Vec<T>) -> Self
        where
            T: Into<ComponentSpecification>,
        {
            self.element_data.child_specs = children.into_iter().map(|x| x.into()).collect();

            self
        }

        #[allow(dead_code)]
        fn extend_children<T>(mut self, children: Vec<T>) -> Self
        where
            T: Into<ComponentSpecification>,
        {
            self.element_data.child_specs.extend(children.into_iter().map(|x| x.into()));

            self
        }

        #[allow(dead_code)]
        fn normal(mut self) -> Self {
            self.element_data.current_state = $crate::elements::element_states::ElementState::Normal;
            self
        }
    };
}

#[macro_export]
macro_rules! generate_component_methods {
    () => {
        $crate::generate_component_methods_no_children!();

        #[allow(dead_code)]
        pub fn push<T>(mut self, component_specification: T) -> Self
        where
            T: Into<ComponentSpecification>,
        {
            self.element_data.child_specs.push(component_specification.into());

            self
        }

        #[allow(dead_code)]
        pub fn push_children<T>(mut self, children: Vec<T>) -> Self
        where
            T: Into<ComponentSpecification>,
        {
            self.element_data.child_specs = children.into_iter().map(|x| x.into()).collect();

            self
        }

        #[allow(dead_code)]
        pub fn extend_children<T>(mut self, children: Vec<T>) -> Self
        where
            T: Into<ComponentSpecification>,
        {
            self.element_data.child_specs.extend(children.into_iter().map(|x| x.into()));

            self
        }

        #[allow(dead_code)]
        pub fn normal(mut self) -> Self {
            self.element_data.current_state = $crate::elements::element_states::ElementState::Normal;
            self
        }
    };
}
