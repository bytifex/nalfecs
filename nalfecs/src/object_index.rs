use crate::{ObjectContainerIndex, ObjectIndexInObjectContainer};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ObjectIndex {
    pub(crate) object_container_index: ObjectContainerIndex,
    pub(crate) object_index_in_object_container: ObjectIndexInObjectContainer,
}

impl ObjectIndex {
    pub(crate) fn new(
        object_container_index: ObjectContainerIndex,
        object_index_in_object_container: ObjectIndexInObjectContainer,
    ) -> Self {
        Self {
            object_container_index,
            object_index_in_object_container,
        }
    }
}
