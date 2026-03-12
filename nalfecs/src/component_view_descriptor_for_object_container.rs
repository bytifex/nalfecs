use crate::ComponentAccessType;

pub type ComponentContainerId = u16;

pub struct ComponentViewDescriptorForObjectContainer {
    object_container_type_id: std::any::TypeId,
    component_container_accesses: Vec<(ComponentContainerId, ComponentAccessType)>,
}

impl ComponentViewDescriptorForObjectContainer {
    pub fn new<ContainerType: 'static>(
        component_container_accesses: Vec<(ComponentContainerId, ComponentAccessType)>,
    ) -> Self {
        Self {
            object_container_type_id: std::any::TypeId::of::<ContainerType>(),
            component_container_accesses,
        }
    }

    pub fn object_container_type_id(&self) -> std::any::TypeId {
        self.object_container_type_id
    }

    pub fn component_container_accesses(&self) -> &[(ComponentContainerId, ComponentAccessType)] {
        &self.component_container_accesses
    }
}
