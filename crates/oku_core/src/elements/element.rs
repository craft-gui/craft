use crate::elements::layout_context::LayoutContext;
use crate::elements::style::Style;
use crate::renderer::renderer::Renderer;
use crate::RenderContext;
use cosmic_text::FontSystem;
use std::any::Any;
use std::fmt::Debug;
use std::sync::Arc;
use taffy::{NodeId, TaffyTree};
use crate::components::component::ComponentOrElement;
use crate::events::Message;

pub trait Element: Any + StandardElementClone + Debug + Send {
    fn children(&self) -> Vec<Box<dyn Element>>;

    fn children_mut(&mut self) -> &mut Vec<Box<dyn Element>>;

    fn name(&self) -> &'static str;

    fn id(&self) -> u64;

    fn key(&self) -> Option<String>;
    fn key_mut(&mut self) -> &mut Option<String>;

    fn tag(&self) -> Option<String>;
    fn tag_mut(&mut self) -> &mut Option<String>;

    fn id_mut(&mut self) -> &mut u64;

    fn draw(&mut self, renderer: &mut Box<dyn Renderer + Send>, render_context: &mut RenderContext);

    fn debug_draw(&mut self, render_context: &mut RenderContext);

    fn compute_layout(&mut self, taffy_tree: &mut TaffyTree<LayoutContext>, font_system: &mut FontSystem) -> NodeId;
    fn finalize_layout(&mut self, taffy_tree: &mut TaffyTree<LayoutContext>, root_node: NodeId, x: f32, y: f32);

    fn computed_style(&self) -> Style;
    fn computed_style_mut(&mut self) -> &mut Style;

    fn in_bounds(&self, x: f32, y: f32) -> bool;
    fn add_update_handler(&mut self, update: Arc<fn(msg: Message, state: Box<dyn Any>, id: u64)>);

    fn as_any(&self) -> &dyn Any;
    
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: Element> From<T> for Box<dyn Element> {
    fn from(element: T) -> Self {
        Box::new(element)
    }
}

impl<T: Element> From<T> for ComponentOrElement {
    fn from(element: T) -> Self {
        ComponentOrElement::Element(Box::new(element))
    }
}

impl From<Box<dyn Element>> for ComponentOrElement {
    fn from(element: Box<dyn Element>) -> Self {
        ComponentOrElement::Element(element)
    }
}

impl dyn Element {

    pub fn print_tree(&self) {
        let mut elements: Vec<(Box<Self>, usize, bool)> = vec![(self.clone_box(), 0, true)];
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
            println!("{}{} ID: {} Tag: {}", prefix, element.name(), element.id(), element.tag().unwrap_or("None".to_string()));
            let children = element.children();
            for (i, child) in children.iter().enumerate() {
                let is_last = i == children.len() - 1;
                elements.push((child.clone(), indent + 1, is_last));
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