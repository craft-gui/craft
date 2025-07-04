use std::collections::HashSet;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;

struct Param {
    name: String,
    type_import: Option<String>,
    type_name: String,
    is_ref: bool,
    is_mut: bool,
}

struct Element {
    type_name: String,
    type_import: String,
}

struct EventHandler {
    name: String,
    params: Vec<Param>,
    elements: Vec<Element>,
}

fn main() {
    let event_handlers = vec![
        EventHandler {
            name: "on_pointer_up".to_string(),
            params: vec![
                Param {
                    name: "press".to_string(),
                    type_import: Some("ui_events::pointer::PointerButtonUpdate".to_string()),
                    type_name: "PointerButtonUpdate".to_string(),
                    is_ref: true,
                    is_mut: false,
                },
            ],
            elements: vec![
                Element {
                    type_name: "Container".to_string(),
                    type_import: "crate::elements::Container".to_string(),
                },
                Element {
                    type_name: "TextInput".to_string(),
                    type_import: "crate::elements::TextInput".to_string(),
                },
                Element {
                    type_name: "Text".to_string(),
                    type_import: "crate::elements::Text".to_string(),
                },
            ],
        },
        EventHandler {
            name: "on_pointer_down".to_string(),
            params: vec![
                Param {
                    name: "press".to_string(),
                    type_import: Some("ui_events::pointer::PointerButtonUpdate".to_string()),
                    type_name: "PointerButtonUpdate".to_string(),
                    is_ref: true,
                    is_mut: false,
                },
            ],
            elements: vec![
                Element {
                    type_name: "Container".to_string(),
                    type_import: "crate::elements::Container".to_string(),
                },
                Element {
                    type_name: "TextInput".to_string(),
                    type_import: "crate::elements::TextInput".to_string(),
                },
                Element {
                    type_name: "Text".to_string(),
                    type_import: "crate::elements::Text".to_string(),
                },
            ],
        },
        EventHandler {
            name: "on_link_clicked".to_string(),
            params: vec![
                Param {
                    name: "press".to_string(),
                    type_import: None,
                    type_name: "str".to_string(),
                    is_ref: true,
                    is_mut: false,
                },
            ],
            elements: vec![
                Element {
                    type_name: "TextInput".to_string(),
                    type_import: "crate::elements::TextInput".to_string(),
                }
            ],
        },
    ];
    generate_event_handler_struct(event_handlers.as_slice());
    generate_element_impls(event_handlers.as_slice());
}

fn generate_event_handler_struct(event_handlers: &[EventHandler]) {
    let path = Path::new("src").join("events").join("event_handlers.rs");
    let mut file = File::create(&path).unwrap();
    writeln!(file, "// This file is generated via build.rs. Do not modify manually!").unwrap();
    writeln!(file).unwrap();
    writeln!(file, "use std::sync::Arc;").unwrap();
    writeln!(file).unwrap();

    writeln!(file, "use crate::components::Event;").unwrap();
    writeln!(file, "use crate::events::Message;").unwrap();
    writeln!(file, "use crate::{{GlobalState, WindowContext}};").unwrap();
    writeln!(file, "use crate::elements::Element;").unwrap();
    writeln!(file, "use crate::reactive::state_store::StateStoreItem;").unwrap();
    writeln!(file).unwrap();


    let mut imports: HashSet<String> = HashSet::new();
    for handler in event_handlers {
        for param in &handler.params {
            if let Some(type_import) = &param.type_import {
                imports.insert(type_import.clone());
            }
        }
    }

    for import in imports {
        writeln!(file, "use {import};").unwrap();
    }

    writeln!(file, "#[allow(clippy::type_complexity)]").unwrap();
    writeln!(file, "#[derive(Clone, Default)]").unwrap();
    writeln!(file, "pub struct EventHandlers {{").unwrap();
    for handler in event_handlers {
        write!(file, "    pub(crate) {}: Option<Arc<dyn Fn(&mut StateStoreItem,
        &mut GlobalState,
        crate::components::Props,
        &mut Event,
        &Message,
        crate::components::component::ComponentId,
        &mut WindowContext,
        Option<&dyn Element>,
        Option<&dyn Element>,", handler.name).unwrap();
        for param in handler.params.iter() {
            if param.is_ref {
                write!(file, "&").unwrap();
            }
            if param.is_mut {
                write!(file, "mut ").unwrap();
            }
            write!(file, " {}", param.type_name).unwrap();
        }
        writeln!(file, ") + Send + Sync>>,").unwrap();
    }
    writeln!(file, "}}").unwrap();

    let _ = Command::new("rustfmt")
        .arg(path)
        .status()
        .expect("Failed to run rustfmt");
}

fn generate_element_impls(event_handlers: &[EventHandler]) {
    let path = Path::new("src").join("elements").join("element_event_impls.rs");
    let mut file = File::create(&path).unwrap();
    writeln!(file, "// This file is generated via build.rs. Do not modify manually!").unwrap();
    writeln!(file).unwrap();

    writeln!(file, "use std::sync::Arc;").unwrap();
    writeln!(file, "use crate::components::Context;").unwrap();
    writeln!(file, "use crate::components::Component;").unwrap();
    writeln!(file).unwrap();

    writeln!(file, "use crate::components::Props;").unwrap();
    writeln!(file, "use crate::components::ComponentId;").unwrap();
    writeln!(file, "use crate::components::Event;").unwrap();
    writeln!(file, "use crate::events::Message;").unwrap();
    writeln!(file, "use crate::{{GlobalState, WindowContext}};").unwrap();
    writeln!(file, "use crate::elements::Element;").unwrap();
    writeln!(file, "use crate::reactive::state_store::StateStoreItem;").unwrap();
    writeln!(file).unwrap();

    let mut imports: HashSet<String> = HashSet::new();
    for handler in event_handlers {
        for element in &handler.elements {
            imports.insert(element.type_import.clone());
        }
        writeln!(file).unwrap();
    }

    for import in imports {
        writeln!(file, "use {import};").unwrap();
    }

    let mut type_imports: HashSet<String> = HashSet::new();
    for handler in event_handlers {
        for param in &handler.params {
            if let Some(type_import) = &param.type_import {
                type_imports.insert(type_import.clone());
            }
        }
    }

    for type_import in type_imports {
        writeln!(file, "use {type_import};").unwrap();
    }
    writeln!(file).unwrap();

    for handler in event_handlers {
        for element in &handler.elements {
            writeln!(file, "impl {} {{", element.type_name).unwrap();
            writeln!(file, "    pub fn {}<ComponentType: Component>(", &handler.name).unwrap();
            writeln!(file, "        mut self,").unwrap();
            write!(file, "        callback: impl Fn(&mut Context<ComponentType>,").unwrap();
            for (param_index, param) in handler.params.iter().enumerate() {
                if param.is_ref {
                    write!(file, "&").unwrap();
                }
                if param.is_mut {
                    write!(file, "mut ").unwrap();
                }
                write!(file, "{}", param.type_name).unwrap();
                if param_index < handler.params.len() - 1 {
                    write!(file, ", ").unwrap();
                }
            }
            writeln!(file, ") + 'static + Send + Sync,").unwrap();
            writeln!(file, "    ) -> Self {{").unwrap();
            write!(file, "        self.element_data_mut().event_handlers.{} = Some(Arc::new(move |state: &mut StateStoreItem,
                  global_state: &mut GlobalState,
                  props: Props,
                  event: &mut Event,
                  message: &Message,
                  id: ComponentId,
                  window_context: &mut WindowContext,
                  target: Option<&dyn Element>,
                  current_target: Option<&dyn Element>,", handler.name).unwrap();
            for param in &handler.params {
                write!(file, "{}: ", param.name).unwrap();
                if param.is_ref {
                    write!(file, "&").unwrap();
                }
                if param.is_mut {
                    write!(file, "mut ").unwrap();
                }
                write!(file, "{}", param.type_name).unwrap();
            }
            writeln!(file, "| {{").unwrap();
            writeln!(file, "            if let Some(casted_state) = state.downcast_mut::<ComponentType>() {{").unwrap();
            writeln!(file, "let mut context = Context::new(
                    None,
                    Some(casted_state),
                    None,
                    Some(global_state),
                    props,
                    None,
                    id,
                    None,
                    Some(window_context),
                    Some(event),
                    Some(message),
                    target,
                    current_target,
                );").unwrap();
            writeln!(file, "                callback(&mut context, ").unwrap();
            for (param_index, param) in handler.params.iter().enumerate() {
                write!(file, "{}", param.name).unwrap();
                if param_index < handler.params.len() - 1 {
                    write!(file, ", ").unwrap();
                }
            }
            write!(file, ");").unwrap();
            writeln!(file, "            }} else {{").unwrap();
            writeln!(file, "                panic!(\"Invalid type passed to callback.\");").unwrap();
            writeln!(file, "            }}").unwrap();
            writeln!(file, "        }}));").unwrap();
            writeln!(file, "        self").unwrap();
            writeln!(file, "    }}").unwrap();
            writeln!(file, "}}").unwrap();
            writeln!(file).unwrap();
        }
    }

    let _ = Command::new("rustfmt")
        .arg(path)
        .status()
        .expect("Failed to run rustfmt");
}