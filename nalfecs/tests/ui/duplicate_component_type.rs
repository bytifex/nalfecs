#[allow(dead_code)]
mod component {
    pub struct Transform(pub String);
}

#[nalfecs::object(container_name = "DuplicateComponentContainer")]
struct DuplicateComponentObject {
    primary: component::Transform,
    secondary: component::Transform,
}

fn main() {}
