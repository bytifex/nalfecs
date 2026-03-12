#[allow(dead_code)]
mod component {
    pub struct Transform(pub String);
    pub type Position = Transform;
}

#[nalfecs::object(container_name = "DuplicateAliasComponentContainer")]
struct DuplicateAliasComponentObject {
    primary: component::Transform,
    secondary: component::Position,
}

fn main() {}
