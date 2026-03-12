use crate::ObjectContainer;

pub trait Object {
    type Container: ObjectContainer + 'static;
}
