use crate::{ComponentViewDescriptorForObjectContainer, ObjectContainerIndex};

pub struct ComponentViewDescriptor {
    pub(crate) view_descriptors: Vec<(
        ObjectContainerIndex,
        ComponentViewDescriptorForObjectContainer,
    )>,
    number_of_components: usize,
}

impl ComponentViewDescriptor {
    pub(crate) fn new(
        view_descriptors: Vec<(
            ObjectContainerIndex,
            ComponentViewDescriptorForObjectContainer,
        )>,
        number_of_components: usize,
    ) -> Self {
        Self {
            view_descriptors,
            number_of_components,
        }
    }

    pub fn number_of_components(&self) -> usize {
        self.number_of_components
    }
}
