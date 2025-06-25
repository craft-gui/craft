use crate::components::{Component, ComponentId, ComponentSpecification};
use crate::components::{Context, Props};
use crate::devtools::dev_tools_colors::CONTAINER_BACKGROUND_COLOR;
use crate::devtools::dev_tools_element::DevTools;
use crate::devtools::layout_window::{LayoutWindow, LayoutWindowProps};
use crate::devtools::tree_window::tree_window;
use crate::elements::element::Element;
use crate::elements::ElementStyles;
use crate::events::{CraftMessage, Message};
use crate::style::Display::Flex;
use crate::style::{FlexDirection, Unit};

#[derive(Default)]
pub(crate) struct DevToolsComponent {
    pub selected_element: Option<ComponentId>,
    pub inspector_hovered_element: Option<ComponentId>,
}

impl Component for DevToolsComponent {
    type GlobalState = ();
    type Props = Option<Box<dyn Element>>;
    type Message = ();

    fn view(context: &mut Context<Self>) -> ComponentSpecification {
        let root = context.props().as_ref().unwrap().clone();
        let element_tree = tree_window(root.as_ref(), context.state().selected_element);

        // Find the selected element in the element tree, so that we can inspect their style values.
        let mut selected_element: Option<&dyn Element> = None;
        if context.state().selected_element.is_some() {
            for element in root.pre_order_iter().collect::<Vec<&dyn Element>>().iter().rev() {
                if element.component_id() != context.state().selected_element.unwrap() {
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
            .push_selected_inspector_element(context.state().selected_element)
            .push_hovered_inspector_element(context.state().inspector_hovered_element)
            .flex_direction(FlexDirection::Column)
            .background(CONTAINER_BACKGROUND_COLOR)
            .width(Unit::Percentage(100.0))
            .height(Unit::Percentage(100.0))
            .max_height(Unit::Percentage(100.0))
            .push(element_tree)
            .push(styles_window)
            .component()
    }

    fn update(context: &mut Context<Self>) {
        if let Some(id) = context.current_target().and_then(|e| e.get_id().clone()) {
            if !id.contains("tree_view_") {
                return;
            }
            
            let id = id.trim_start_matches("tree_view_").to_owned();
            
            // Set the selected element in the element tree inspector.
            if context.message().clicked() {
                let component_id: ComponentId = id.parse().unwrap();
                context.state_mut().selected_element = Some(component_id);
            }

            // Update the hovered element in the inspector tree, so that the DevTools widget can draw a debug overlay.
            if let Message::CraftMessage(CraftMessage::PointerMovedEvent(_pointer_moved_event)) = context.message() {
                let component_id: ComponentId = id.parse().unwrap();
                context.state_mut().inspector_hovered_element = Some(component_id);
            }
        }
    }
}

pub fn dev_tools_view(root: Box<dyn Element>) -> ComponentSpecification {
    DevToolsComponent::component().props(Props::new(Some(root)))
}
