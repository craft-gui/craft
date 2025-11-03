use crate::elements::element_data::ElementData;
use crate::elements::scroll_state::ScrollState;
use crate::layout::layout_context::LayoutContext;
use crate::text::text_context::TextContext;
use craft_primitives::geometry::{Rectangle, Size};
use craft_renderer::RenderList;
use kurbo::{Affine, Point};
use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::sync::Arc;
use taffy::{NodeId, TaffyTree};
use winit::window::Window;
use crate::elements::core::{resolve_clip_for_scrollable, ElementInternals};
use crate::elements::core::ElementData as ElementDataTrait;
use crate::elements::Element;
use crate::events::{CraftMessage, Event};

pub struct Container {
    element_data: ElementData,
    me: Option<Weak<RefCell<Container>>>,
}

impl Container {
    pub fn new() -> Rc<RefCell<Self>> {
        let me = Rc::new(RefCell::new(Self {
            element_data: ElementData::new(true),
            me: None,
        }));
        me.borrow_mut().me = Some(Rc::downgrade(&me.clone()));
        me
    }
}

impl crate::elements::core::ElementData for Container {
    fn element_data(&self) -> &ElementData {
        &self.element_data
    }

    fn element_data_mut(&mut self) -> &mut ElementData {
        &mut self.element_data
    }

    fn push(&mut self, child: Rc<RefCell<dyn Element>>) {
        let me: Weak<RefCell<dyn Element>> = self.me.clone().unwrap() as Weak<RefCell<dyn Element>>;
        child.borrow_mut().element_data_mut().parent = Some(me);
        self.element_data.children.push(child);
    }
}

impl Element for Container {
}

impl ElementInternals for Container {
    fn compute_layout(&mut self, taffy_tree: &mut TaffyTree<LayoutContext>, scale_factor: f64) -> Option<NodeId> {
        self.element_data.layout_item.child_nodes.clear();

        for child in &mut self.element_data.children {
            let child_node = child.borrow_mut().compute_layout(taffy_tree, scale_factor);
            self.element_data.layout_item.push_child(&child_node);
        }

        let current_style = self.element_data.style.to_taffy_style();

        self.element_data.layout_item.build_tree(taffy_tree, current_style)
    }

    fn finalize_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        root_node: NodeId,
        position: Point,
        z_index: &mut u32,
        transform: Affine,
        pointer: Option<Point>,
        text_context: &mut TextContext,
        clip_bounds: Option<Rectangle>,
    ) {
        let result = taffy_tree.layout(root_node).unwrap();
        self.resolve_box(position, transform, result, z_index);
        self.finalize_borders();

        self.element_data.layout_item.scrollbar_size =
            Size::new(result.scrollbar_size.width, result.scrollbar_size.height);
        self.element_data.layout_item.computed_scrollbar_size =
            Size::new(result.scroll_width(), result.scroll_height());
        self.element_data.finalize_scroll();
        let scroll_y = self.element_data.scroll_state.as_ref().unwrap().scroll_y;
        self.resolve_clip(clip_bounds);
        let child_transform = Affine::translate((0.0, -scroll_y as f64));

        for child in self.element_data.children.iter_mut() {
            let taffy_child_node_id = child.borrow().element_data().layout_item.taffy_node_id;
            if taffy_child_node_id.is_none() {
                continue;
            }

            child.borrow_mut().finalize_layout(
                taffy_tree,
                taffy_child_node_id.unwrap(),
                self.element_data.layout_item.computed_box.position,
                z_index,
                transform * child_transform,
                pointer,
                text_context,
                self.element_data.layout_item.clip_bounds,
            );
        }
    }

    fn draw(
        &mut self,
        renderer: &mut RenderList,
        text_context: &mut TextContext,
        pointer: Option<Point>,
        window: Option<Arc<Window>>,
        scale_factor: f64,
    ) {

        let current_style = self.element_data.current_style();

        if !current_style.visible() {
            return;
        }

        // We draw the borders before we start any layers, so that we don't clip the borders.
        self.draw_borders(renderer, scale_factor);
        self.maybe_start_layer(renderer, scale_factor);
        for child in self.children() {
            child.borrow_mut().draw(renderer, text_context, pointer, window.clone(), scale_factor);
        }
        self.maybe_end_layer(renderer);

        self.draw_scrollbar(renderer, scale_factor);
    }

    fn on_event(
        &mut self,
        message: &CraftMessage,
        _text_context: &mut TextContext,
        should_style: bool,
        event: &mut Event,
        target: Option<Rc<RefCell<dyn ElementInternals>>>,
        //_current_target: Option<&dyn Element>,
    ) {
        //self.on_style_event(message, should_style, event);
        //self.maybe_unset_focus(message, event, target);

        self.element_data.on_event(message, event);
    }

    fn resolve_clip(&mut self, clip_bounds: Option<Rectangle>) {
        resolve_clip_for_scrollable(self, clip_bounds);
    }

}
