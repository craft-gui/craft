use crate::components::Props;
use crate::components::{Component, ComponentId, ComponentSpecification, Event};
use crate::devtools::dev_tools_colors::CONTAINER_BACKGROUND_COLOR;
use crate::devtools::dev_tools_element::DevTools;
use crate::devtools::tree_window::tree_window;
use crate::devtools::layout_window::{LayoutWindow, LayoutWindowProps};
use crate::elements::element::Element;
use crate::elements::ElementStyles;
use crate::events::{CraftMessage, Message};
use crate::style::Display::Flex;
use crate::style::{FlexDirection, Unit};
use crate::window_context::WindowContext;

#[derive(Default)]
pub(crate) struct DevToolsComponent {
    pub selected_element: Option<ComponentId>,
    pub inspector_hovered_element: Option<ComponentId>,
}

impl Component for DevToolsComponent {
    type GlobalState = ();
    type Props = Option<Box<dyn Element>>;
    type Message = ();

    fn view(
        &self,
        _global_state: &Self::GlobalState,
        props: &Self::Props,
        _children: Vec<ComponentSpecification>,
        _id: ComponentId,
        _window: &WindowContext,
    ) -> ComponentSpecification {
        let root = props.as_ref().unwrap().clone();
        let element_tree = tree_window(root.as_ref(), self.selected_element);

        // Find the selected element in the element tree, so that we can inspect their style values.
        let mut selected_element: Option<&dyn Element> = None;
        if self.selected_element.is_some() {
            for element in root.pre_order_iter().collect::<Vec<&dyn Element>>().iter().rev() {
                if element.component_id() != self.selected_element.unwrap() {
                    continue;
                }

                selected_element = Some(*element);
                break;
            }
        }

        let styles_window = LayoutWindow::component().props(Props::new(LayoutWindowProps {
            selected_element: selected_element.map(|e| e.clone_box()),
        }));

        DevTools::new()
            .display(Flex)
            .push_debug_inspector_tree(root)
            .push_selected_inspector_element(self.selected_element)
            .push_hovered_inspector_element(self.inspector_hovered_element)
            .flex_direction(FlexDirection::Column)
            .background(CONTAINER_BACKGROUND_COLOR)
            .width(Unit::Percentage(100.0))
            .height(Unit::Percentage(100.0))
            .max_height(Unit::Percentage(100.0))
            .push(element_tree)
            .push(styles_window)
            .component()
    }

    fn update(
        &mut self,
        _global_state: &mut Self::GlobalState,
        _props: &Self::Props,
        event: &mut Event,
        message: &Message,
    ) {
        if let Some(id) = event.current_target.and_then(|e| e.get_id().clone()) {
            if !id.contains("tree_view_") {
                return;
            }
            
            let id = id.trim_start_matches("tree_view_").to_owned();
            
            // Set the selected element in the element tree inspector.
            if message.clicked() {
                let component_id: ComponentId = id.parse().unwrap();
                self.selected_element = Some(component_id);
            }

            // Update the hovered element in the inspector tree, so that the DevTools widget can draw a debug overlay.
            if let Message::CraftMessage(CraftMessage::PointerMovedEvent(_pointer_moved_event)) = message {
                let component_id: ComponentId = id.parse().unwrap();
                self.inspector_hovered_element = Some(component_id);
            }
        }
    }
}

pub fn dev_tools_view(root: Box<dyn Element>) -> ComponentSpecification {
    DevToolsComponent::component().props(Props::new(Some(root)))
}
