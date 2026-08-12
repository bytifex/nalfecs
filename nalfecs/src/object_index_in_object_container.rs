use crate::{ComponentIndex, GenerationalSlotMapIndex};

/// Basically this is the same as `ObjectIndexInObjectContainer`. The underlying
/// data should have the same value for every components in the same object.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ObjectIndexInObjectContainer(pub(crate) GenerationalSlotMapIndex);

impl From<ComponentIndex> for ObjectIndexInObjectContainer {
    fn from(value: ComponentIndex) -> Self {
        Self(value.0)
    }
}

impl From<ObjectIndexInObjectContainer> for ComponentIndex {
    fn from(value: ObjectIndexInObjectContainer) -> Self {
        Self(value.0)
    }
}
