use crate::components::component::{ComponentOrElement, ComponentSpecification};
use crate::components::UpdateResult;
use crate::elements::common_element_data::CommonElementData;
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
use cosmic_text::FontSystem;
use taffy::{NodeId, Overflow, Position, TaffyTree};

#[derive(Clone, Debug)]
pub struct ElementBox {
    pub(crate) internal: Box<dyn Element>,
}

pub(crate) trait Element: Any + StandardElementClone + Debug + Send + Sync {
    fn common_element_data(&self) -> &CommonElementData;
    fn common_element_data_mut(&mut self) -> &mut CommonElementData;

    fn children(&self) -> Vec<&dyn Element> {
        self.common_element_data().children.iter().map(|x| x.internal.as_ref()).collect()
    }

    fn children_mut(&mut self) -> &mut Vec<ElementBox> {
        &mut self.common_element_data_mut().children
    }

    fn style(&self) -> &Style {
        &self.common_element_data().style
    }

    fn style_mut(&mut self) -> &mut Style {
        &mut self.common_element_data_mut().style
    }

    fn in_bounds(&self, point: Point) -> bool {
        let common_element_data = self.common_element_data();

        let transformed_border_rectangle =
            common_element_data.computed_layered_rectangle_transformed.border_rectangle();

        transformed_border_rectangle.contains(&point)
    }

    fn get_id(&self) -> &Option<String> {
        &self.common_element_data().id
    }

    #[allow(dead_code)]
    fn id(&mut self, id: Option<&str>) -> Box<dyn Element> {
        self.common_element_data_mut().id = id.map(String::from);
        self.clone_box()
    }

    fn component_id(&self) -> u64 {
        self.common_element_data().component_id
    }

    fn taffy_node_id(&self) -> Option<NodeId> {
        self.common_element_data().taffy_node_id
    }

    fn set_component_id(&mut self, id: u64) {
        self.common_element_data_mut().component_id = id;
    }

    fn name(&self) -> &'static str;

    fn draw(
        &mut self,
        renderer: &mut RendererBox,
        font_system: &mut FontSystem,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        root_node: NodeId,
        element_state: &ElementStateStore,
        pointer: Option<Point>,
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
    /// Store the computed values in the `common_element_data` struct.
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

    fn resolve_layer_rectangle(
        &mut self,
        relative_position: Point,
        scroll_transform: glam::Mat4,
        result: &taffy::Layout,
        layout_order: &mut u32,
    ) {
        let common_element_data_mut = self.common_element_data_mut();
        common_element_data_mut.layout_order = *layout_order;
        *layout_order += 1;

        let position = match common_element_data_mut.style.position() {
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
        if common_element_data_mut.style.position() == Position::Absolute {
            size = Size::new(f32::max(result.size.width, result.content_size.width), f32::max(result.size.height, result.content_size.height));
        }
        
        common_element_data_mut.computed_border_rectangle_overflow_size =
            Size::new(result.content_size.width, result.content_size.height);
        common_element_data_mut.computed_layered_rectangle = ElementRectangle {
            margin: Margin::new(result.margin.top, result.margin.right, result.margin.bottom, result.margin.left),
            border: Border::new(result.border.top, result.border.right, result.border.bottom, result.border.left),
            padding: Padding::new(result.padding.top, result.padding.right, result.padding.bottom, result.padding.left),
            position,
            size,
        };
        common_element_data_mut.computed_layered_rectangle_transformed =
            common_element_data_mut.computed_layered_rectangle.transform(scroll_transform);
    }

    fn draw_children(
        &mut self,
        renderer: &mut RendererBox,
        font_system: &mut FontSystem,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        element_state: &ElementStateStore,
        pointer: Option<Point>,
    ) {
        for child in self.common_element_data_mut().children.iter_mut() {
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
            );
        }
    }

    fn draw_borders(&self, renderer: &mut RendererBox) {
        let common_element_data = self.common_element_data();
        let current_style = common_element_data.current_style();
        let background_color = current_style.background();

        // OPTIMIZATION: Draw a normal rectangle if no border values have been modified.
        if !current_style.has_border() {
            renderer.draw_rect(
                common_element_data.computed_layered_rectangle_transformed.padding_rectangle(),
                background_color,
            );
            return;
        }

        let computed_border_spec = &common_element_data.computed_border;

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
        let common_data = self.common_element_data();

        common_data.current_style().overflow()[1] == Overflow::Scroll
    }

    fn maybe_start_layer(&self, renderer: &mut RendererBox) {
        let common_data = self.common_element_data();
        let padding_rectangle = common_data.computed_layered_rectangle_transformed.padding_rectangle();

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
        let common_element_data = self.common_element_data_mut();

        // OPTIMIZATION: Don't compute the border if no border style values have been modified.
        if !common_element_data.current_style().has_border() {
            return;
        }

        let element_rect = common_element_data.computed_layered_rectangle_transformed;
        let borders = element_rect.border;
        let border_spec = BorderSpec::new(
            element_rect.border_rectangle(),
            [borders.top, borders.right, borders.bottom, borders.left],
            common_element_data.current_style().border_radius(),
            common_element_data.current_style().border_color(),
        );
        common_element_data.computed_border = border_spec.compute_border_spec();
    }

    fn finalize_scrollbar(&mut self, scroll_y: f32) {
        let common_element_data = self.common_element_data_mut();

        if common_element_data.style.overflow()[0] != Overflow::Scroll {
            return;
        }

        let computed_layered_rectangle_transformed = common_element_data.computed_layered_rectangle_transformed;
        let padding_rectangle = computed_layered_rectangle_transformed.padding_rectangle();

        let computed_content_height = common_element_data.computed_border_rectangle_overflow_size.height;

        let client_height = padding_rectangle.height;
        let scroll_height = computed_content_height - computed_layered_rectangle_transformed.border.top;

        let scrolltrack_width = common_element_data.scrollbar_size.width;
        let scrolltrack_height = client_height;

        let max_scroll_y = (scroll_height - client_height).max(0.0);
        common_element_data.max_scroll_y = max_scroll_y;

        let visible_y = client_height / scroll_height;
        let scrollthumb_height = scrolltrack_height * visible_y;
        let remaining_height = scrolltrack_height - scrollthumb_height;
        let scrollthumb_offset = if max_scroll_y != 0.0 { scroll_y / max_scroll_y * remaining_height } else { 0.0 };

        common_element_data.computed_scroll_track = Rectangle::new(
            computed_layered_rectangle_transformed.position.x + computed_layered_rectangle_transformed.size.width
                - scrolltrack_width
                - computed_layered_rectangle_transformed.border.right,
            computed_layered_rectangle_transformed.position.y + computed_layered_rectangle_transformed.border.top,
            scrolltrack_width,
            scrolltrack_height,
        );

        let scrollthumb_width = scrolltrack_width;

        common_element_data.computed_scroll_thumb = Rectangle::new(
            computed_layered_rectangle_transformed.position.x + computed_layered_rectangle_transformed.size.width
                - scrolltrack_width
                - computed_layered_rectangle_transformed.border.right,
            computed_layered_rectangle_transformed.position.y
                + computed_layered_rectangle_transformed.border.top
                + scrollthumb_offset,
            scrollthumb_width,
            scrollthumb_height,
        );
    }

    /// Called when the element is assigned a unique component id.
    fn initialize_state(&self, _font_system: &mut FontSystem) -> ElementStateStoreItem {
        ElementStateStoreItem {
            base: Default::default(),
            data: Box::new(()),
        }
    }

    #[allow(dead_code)]
    fn finalize_state(&mut self, element_state: &mut ElementStateStore, pointer: Option<Point>) {
        let common_element_data = self.common_element_data_mut();
        let element_state = element_state.storage.get_mut(&common_element_data.component_id).unwrap();
        element_state.base.current_state = ElementState::Normal;

        let border_rectangle = common_element_data.computed_layered_rectangle_transformed.border_rectangle();

        if let Some(pointer) = pointer {
            if border_rectangle.contains(&pointer) {
                element_state.base.current_state = ElementState::Hovered;
            }
        }
    }

    fn get_base_state<'a>(&self, element_state: &'a ElementStateStore) -> &'a ElementStateStoreItem {
        element_state.storage.get(&self.common_element_data().component_id).unwrap()
    }

    #[allow(dead_code)]
    fn get_base_state_mut<'a>(&self, element_state: &'a mut ElementStateStore) -> &'a mut ElementStateStoreItem {
        element_state.storage.get_mut(&self.common_element_data().component_id).unwrap()
    }

    /// Called on sequential renders to update any state that the element may have.
    fn update_state(&self, _font_system: &mut FontSystem, _element_state: &mut ElementStateStore, _reload_fonts: bool) {}

    fn default_style(&self) -> Style {
        Style::default()
    }

    fn merge_default_style(&mut self) {
        self.common_element_data_mut().style = Style::merge(&self.default_style(), &self.common_element_data().style);
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
        let key = element.internal.common_element_data().key.clone();
        let children = element.internal.common_element_data().child_specs.clone();
        let props = element.internal.common_element_data().props.clone();
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
        let key = element.common_element_data().key.clone();
        let children_specs = element.common_element_data().child_specs.clone();
        let props = element.common_element_data().props.clone();
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
            self.common_element_data.key = Some(key.to_string());

            self
        }

        #[allow(dead_code)]
        pub fn props(mut self, props: Props) -> Self {
            self.common_element_data.props = Some(props);

            self
        }

        #[allow(dead_code)]
        pub fn id(mut self, id: &str) -> Self {
            self.common_element_data.id = Some(id.to_string());
            self
        }

        #[allow(dead_code)]
        pub fn hovered(mut self) -> Self {
            self.common_element_data.current_state = $crate::elements::element_states::ElementState::Hovered;
            self
        }

        #[allow(dead_code)]
        pub fn pressed(mut self) -> Self {
            self.common_element_data.current_state = $crate::elements::element_states::ElementState::Pressed;
            self
        }

        #[allow(dead_code)]
        pub fn disabled(mut self) -> Self {
            self.common_element_data.current_state = $crate::elements::element_states::ElementState::Disabled;
            self
        }

        #[allow(dead_code)]
        pub fn focused(mut self) -> Self {
            self.common_element_data.current_state = $crate::elements::element_states::ElementState::Focused;
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
            self.common_element_data.child_specs.push(component_specification.into());

            self
        }

        #[allow(dead_code)]
        fn push_children<T>(mut self, children: Vec<T>) -> Self
        where
            T: Into<ComponentSpecification>,
        {
            self.common_element_data.child_specs = children.into_iter().map(|x| x.into()).collect();

            self
        }

        #[allow(dead_code)]
        fn extend_children<T>(mut self, children: Vec<T>) -> Self
        where
            T: Into<ComponentSpecification>,
        {
            self.common_element_data.child_specs.extend(children.into_iter().map(|x| x.into()));

            self
        }

        #[allow(dead_code)]
        fn normal(mut self) -> Self {
            self.common_element_data.current_state = $crate::elements::element_states::ElementState::Normal;
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
            self.common_element_data.child_specs.push(component_specification.into());

            self
        }

        #[allow(dead_code)]
        pub fn push_children<T>(mut self, children: Vec<T>) -> Self
        where
            T: Into<ComponentSpecification>,
        {
            self.common_element_data.child_specs = children.into_iter().map(|x| x.into()).collect();

            self
        }

        #[allow(dead_code)]
        pub fn extend_children<T>(mut self, children: Vec<T>) -> Self
        where
            T: Into<ComponentSpecification>,
        {
            self.common_element_data.child_specs.extend(children.into_iter().map(|x| x.into()));

            self
        }

        #[allow(dead_code)]
        pub fn normal(mut self) -> Self {
            self.common_element_data.current_state = $crate::elements::element_states::ElementState::Normal;
            self
        }
    };
}
