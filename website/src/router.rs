use crate::docs::docs_component::Docs;
use crate::examples::Examples;
use crate::index::index_page;
use craft::components::{Component, ComponentSpecification};
use craft::WindowContext;

#[derive(Clone)]
pub(crate) struct MappedPath<'a> {
    pub(crate) path: &'a str,
    pub(crate) component_specification: ComponentSpecification,
}

impl<'a> MappedPath<'a> {
    pub(crate) fn new(path: &'a str, component_specification: ComponentSpecification) -> Self {
        MappedPath {
            path,
            component_specification,
        }
    }
}

pub fn resolve_route<'a>(path: &'a str, window_ctx: &'a WindowContext) -> Option<MappedPath<'a>> {
    let mapped_paths: Vec<MappedPath> = vec![
        MappedPath::new("/examples/*", Examples::component().key("examples")),
        MappedPath::new("/docs/*", Docs::component().key("docs")),
        MappedPath::new("/*", index_page(window_ctx).key("index")),
    ];
    for mapped_path in &mapped_paths {
        let mut matches = true;
        for (path_resource, rule_token) in path.split("/").zip(mapped_path.path.split("/")) {
            if rule_token == "*" {
                continue;
            }

            if rule_token != path_resource {
                matches = false;
                break;
            }
        }

        if matches {
            return Some(mapped_path.clone());
        }
    }

    None
}
