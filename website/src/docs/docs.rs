use crate::docs::hello_world::HelloWorldPage;
use crate::docs::how_to_contribute::HowToContributePage;
use crate::docs::installation::InstallationPage;
use crate::docs::state_management::StateManagementPage;
use crate::docs::styling::StylingPage;
use crate::theme::{wrapper, ACTIVE_LINK_COLOR, DEFAULT_LINK_COLOR, MAX_DOCS_CONTENT_WIDTH, MOBILE_MEDIA_QUERY_WIDTH};
use crate::WebsiteGlobalState;
use craft::components::{Component, ComponentId, ComponentSpecification, Event};
use craft::elements::{Container, ElementStyles, Text};
use craft::events::ui_events::pointer::PointerButtonUpdate;
use craft::style::{Display, FlexDirection, JustifyContent, Overflow, Unit, Weight};
use craft::{palette, WindowContext};

#[derive(Default)]
pub(crate) struct Docs {}

fn docs_menu_header(text: &str) -> Text {
    Text::new(text.to_uppercase().as_str())
        .font_size(14.0)
        .font_weight(Weight::BOLD)
        .padding("15px", "0px", "20px", "0px")
}

fn docs_menu_link(label: &str, link: &str, is_current: bool) -> Text {
    let href = link.to_string();
    let mut text = Text::new(label)
        .color(DEFAULT_LINK_COLOR)
        .on_pointer_button_up(
            move |_state: &mut Docs, global_state: &mut WebsiteGlobalState, event: &mut Event, pointer_button: &PointerButtonUpdate| {
                if pointer_button.is_primary() {
                    global_state.set_route(href.as_str());
                    event.prevent_propagate();
                }
            },
        )
        .id(link)
        .disable_selection();
    if is_current {
        text = text.color(ACTIVE_LINK_COLOR);
    }
    text
}

fn docs_menu_separator() -> Container {
    Container::new()
        .margin("34px", "0px", "12px", "0px")
        .height("1px")
        .width("100%").background(palette::css::LIGHT_GRAY)
}

fn docs_menu(docs_menu_section: &Vec<DocsMenuSection>, current_route: &str) -> Container {
    let mut container = Container::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column);
    
    for (index, section) in docs_menu_section.iter().enumerate() {
        let is_first = index == 0;
        let is_last = index == docs_menu_section.len() - 1;

        let mut top_padding = "15px";
        if is_first {
            top_padding = "0px";
        }
        container.push_in_place(docs_menu_header(section.section_name.as_str()).padding(top_padding, "0px", "20px", "0px").component());

        for link in &section.links {
            container.push_in_place(docs_menu_link(link.label.as_str(), link.href.as_str(), current_route == link.href).component());
        }

        if !is_last {
            container.push_in_place(docs_menu_separator().component());
        }
    }

    container
}

struct DocsMenuSection {
    section_name: String,
    links: Vec<Link>,
}

#[derive(Clone)]
#[derive(Debug)]
struct Link {
    label: String,
    href: String,
}

impl Link {
    pub(crate) fn new(label: &str, href: &str) -> Link {
        Link {
            label: label.to_string(),
            href: href.to_string(),
        }
    }
}

impl Component for Docs {
    type Props = ();
    type GlobalState = WebsiteGlobalState;
    type Message = ();

    fn view(
        &self,
        global_state: &WebsiteGlobalState,
        _props: &Self::Props,
        _children: Vec<ComponentSpecification>,
        _id: ComponentId,
        window: &WindowContext,
    ) -> ComponentSpecification {
        let mut route = global_state.get_route();
        if route == "/docs" {
            route = "/docs/installation".to_string();
        }
        
        let container = Container::new()
            .overflow_y(Overflow::Scroll)
            .width("100%")
            .padding("50px", "0px", "0px", "0px")
            ;

        let mut wrapper = wrapper().display(Display::Flex).gap("100px");

        if window.window_width() <= MOBILE_MEDIA_QUERY_WIDTH {
            wrapper = wrapper.flex_direction(FlexDirection::ColumnReverse);
        }

        let menu_sections = vec![
            DocsMenuSection {
                section_name: "Quick Start".to_string(),
                links: vec![
                    Link::new("Installation", "/docs/installation"),
                    Link::new("Hello World", "/docs/hello_world"),
                    Link::new("State Management", "/docs/state_management"),
                    Link::new("Styling", "/docs/styling"),
                ],
            },
            DocsMenuSection {
                section_name: "Contributing".to_string(),
                links: vec![
                    Link::new("How to Contribute", "/docs/how_to_contribute")
                ],
            }
        ];

        let all_links: Vec<Link> = menu_sections.iter().flat_map(|sec| sec.links.clone()).collect();

        let mut current_route_index: usize = 0;
        for (index, link) in all_links.iter().enumerate() {
            if link.href == route {
                current_route_index = index;
            }
        }

        let prev_link: Option<Link> = if current_route_index as i64 - 1 >= 0 && all_links.len() > 0 {
            Some(all_links[current_route_index - 1].clone())
        } else {
            None
        };

        let next_link: Option<Link> = if current_route_index + 1 <= all_links.len() - 1  {
            Some(all_links[current_route_index + 1].clone())
        } else {
            None
        };

        let current_component = match route.as_str() {
            "/docs/installation" => InstallationPage::component().key("installation_page"),
            "/docs/hello_world" => HelloWorldPage::component().key("hello_world_page"),
            "/docs/state_management" => StateManagementPage::component().key("state_management_page"),
            "/docs/styling" => StylingPage::component().key("styling_page"),
            "/docs/how_to_contribute" => HowToContributePage::component().key("how_to_contribute_page"),
            _ => InstallationPage::component().key("installation_page")
        };

        let wrapper = wrapper.push(docs_menu(&menu_sections, route.as_str()));

        let mut content_prev_next_bar = Container::new()
            .display(Display::Flex)
            .justify_content(JustifyContent::SpaceBetween)
            .width("100%")
            .margin(Unit::Auto, Unit::Px(0.0), Unit::Px(0.0), Unit::Px(0.0))
            .padding("40px", "0px", "40px", "0px")
            ;

        if let Some(prev_link) = prev_link {
            content_prev_next_bar = content_prev_next_bar.push(docs_menu_link("Previous", prev_link.href.as_str(), true));
        }

        if let Some(next_link) = next_link {
            content_prev_next_bar = content_prev_next_bar.push(
                docs_menu_link("Next", next_link.href.as_str(), true)
                    .margin("0px", "0px", "0px", "auto")
            );
        }

        let content_wrapper = Container::new()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .width("100%")
            .max_width(Unit::Px(MAX_DOCS_CONTENT_WIDTH))
            .push(current_component)
            .push(content_prev_next_bar);

        let wrapper = wrapper.push(content_wrapper);

        let container = container.push(wrapper);

        container.component()
    }
}
