use crate::GenerationalSlotMapIndex;

/// Index of an `ObjectContainer` inside `Container`
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ObjectContainerIndex(pub(crate) GenerationalSlotMapIndex);
