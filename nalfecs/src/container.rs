use std::{any::TypeId, collections::HashMap, sync::Arc};

use crate::{
    ComponentAccess, ComponentViewDescriptor, ComponentViewIterator, GenerationalSlotMap, Object,
    ObjectContainer, ObjectContainerFor, ObjectContainerIndex, ObjectIndex,
};

#[derive(Clone)]
pub struct Container {
    object_containers: Arc<GenerationalSlotMap<Box<dyn ObjectContainer>>>,
    object_container_indices_by_type: Arc<HashMap<TypeId, ObjectContainerIndex>>,
}

impl Container {
    pub fn new(containers: impl IntoIterator<Item = Box<dyn ObjectContainer>>) -> Self {
        let mut object_containers = GenerationalSlotMap::default();
        let mut object_container_indices_by_type = HashMap::new();

        for container in containers {
            let object_container_type = container.as_any().type_id();
            let object_container_index =
                ObjectContainerIndex(object_containers.create_object(container));

            // Keep the first container for a type to match previous first_index behavior.
            object_container_indices_by_type
                .entry(object_container_type)
                .or_insert(object_container_index);
        }

        Container {
            object_containers: Arc::new(object_containers),
            object_container_indices_by_type: Arc::new(object_container_indices_by_type),
        }
    }

    pub fn view_descriptor(
        &self,
        component_accesses: &[ComponentAccess],
    ) -> ComponentViewDescriptor {
        let view_descriptors = self
            .object_containers
            .iter()
            .filter_map(|(index, container)| {
                Some((
                    ObjectContainerIndex(index),
                    container.view_descriptor(component_accesses)?,
                ))
            })
            .collect();

        ComponentViewDescriptor::new(view_descriptors, component_accesses.len())
    }

    pub fn iter_object_container_view_iters<'a>(
        &'a self,
        view_desc: &ComponentViewDescriptor,
    ) -> impl Iterator<Item = ComponentViewIterator<'a>> {
        view_desc
            .view_descriptors
            .iter()
            .filter_map(|(container_index, view_desc)| {
                let container = self.object_containers.get_ref(container_index.0)?;
                container
                    .iter_for(view_desc)
                    .map(|iter| iter.with_object_container_index(*container_index))
            })
    }

    pub fn component_iter<'a, T: 'static>(&'a self) -> Box<dyn Iterator<Item = &'a T> + 'a> {
        let view_desc = self.view_descriptor(&[ComponentAccess::immutable::<T>()]);
        let items: Vec<&'a T> = self
            .iter_object_container_view_iters(&view_desc)
            .flat_map(|iter| {
                let container = iter.component_container_unchecked::<T>(0);
                container.iter().collect::<Vec<_>>()
            })
            .collect();
        Box::new(items.into_iter())
    }

    pub fn component_iter_mut<'a, T: 'static>(
        &'a self,
    ) -> Box<dyn Iterator<Item = &'a mut T> + 'a> {
        let view_desc = self.view_descriptor(&[ComponentAccess::mutable::<T>()]);
        let items: Vec<&'a mut T> = self
            .iter_object_container_view_iters(&view_desc)
            .flat_map(|mut iter| {
                let container = iter.component_container_mut_unchecked::<T>(0);
                container.iter_mut().collect::<Vec<_>>()
            })
            .collect();
        Box::new(items.into_iter())
    }

    pub fn add<T>(&self, object: T) -> Option<ObjectIndex>
    where
        T: Object + 'static,
        T::Container: ObjectContainerFor<T>,
    {
        let object_container_index = *self
            .object_container_indices_by_type
            .get(&TypeId::of::<T::Container>())?;
        let container = self.object_containers.get_ref(object_container_index.0)?;
        let typed_container = container.as_any().downcast_ref::<T::Container>()?;

        let object_in_object_container = typed_container.add_object(object);
        Some(ObjectIndex::new(
            object_container_index,
            object_in_object_container,
        ))
    }

    pub fn remove<T>(&self, index: ObjectIndex) -> Option<T>
    where
        T: Object + 'static,
        T::Container: ObjectContainerFor<T>,
    {
        let container = self
            .object_containers
            .get_ref(index.object_container_index.0)?;
        let typed_container = container.as_any().downcast_ref::<T::Container>()?;
        typed_container.remove_object(index.object_index_in_object_container)
    }
}
