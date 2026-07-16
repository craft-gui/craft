#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ResourceType {
    Image,
    Font,
    TinyVg,
    Other(String)
}