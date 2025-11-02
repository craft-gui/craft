use std::cell::RefCell;
use std::rc::Rc;
use kurbo::{Affine, Point};
use taffy::{NodeId, TaffyTree};
use craft_primitives::geometry::Rectangle;
use crate::elements::element::Element;
use crate::elements::element_data::ElementData;
use crate::layout::layout_context::LayoutContext;
use crate::text::text_context::TextContext;

pub struct Button {
    element_data: ElementData,
    on_click: Vec<Box<dyn FnMut()>>
}

impl Button {
    pub fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            element_data: ElementData::default(),
            on_click: Vec::new(),
        }))
    }

    pub fn push(&mut self, child: Rc<RefCell<dyn Element>>) {
        self.children_mut().push(child)
    }

    pub fn on_click(&mut self, on_click: Box<dyn FnMut() -> ()>) {
        self.on_click.push(on_click);
    }

    pub fn click(&mut self) {
        for on_click in &mut self.on_click {
            on_click();
        }
    }
}

impl Element for Button {
    fn element_data(&self) -> &ElementData {
        &self.element_data
    }

    fn element_data_mut(&mut self) -> &mut ElementData {
        &mut self.element_data
    }

    fn compute_layout(&mut self, taffy_tree: &mut TaffyTree<LayoutContext>, scale_factor: f64) -> Option<NodeId> {
        None
    }

    fn finalize_layout(&mut self, taffy_tree: &mut TaffyTree<LayoutContext>, root_node: NodeId, position: Point, z_index: &mut u32, transform: Affine, pointer: Option<Point>, text_context: &mut TextContext, clip_bounds: Option<Rectangle>) {
    }
}