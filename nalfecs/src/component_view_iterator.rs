use std::any::Any;

use parking_lot::RwLock;

use crate::{ComponentContainer, ObjectContainerIndex, ObjectIndex, ObjectIndexInObjectContainer};

pub enum ComponentContainerGuard<'a> {
    Immutable(parking_lot::MappedRwLockReadGuard<'a, dyn Any>),
    Mutable(parking_lot::MappedRwLockWriteGuard<'a, dyn Any>),
}

impl<'a> ComponentContainerGuard<'a> {
    pub fn immutable<T: 'static>(rwlock: &'a RwLock<T>) -> Self {
        let guard = parking_lot::RwLockReadGuard::map(rwlock.read(), |value| value as &dyn Any);
        Self::Immutable(guard)
    }

    pub fn mutable<T: 'static>(rwlock: &'a RwLock<T>) -> Self {
        let guard =
            parking_lot::RwLockWriteGuard::map(rwlock.write(), |value| value as &mut dyn Any);
        Self::Mutable(guard)
    }

    pub(crate) fn as_immutable<ComponentType: 'static>(
        &self,
    ) -> Option<&'a ComponentContainer<ComponentType>> {
        match self {
            ComponentContainerGuard::Immutable(guard) => guard
                .downcast_ref::<ComponentContainer<ComponentType>>()
                .map(|container| unsafe {
                    std::mem::transmute::<
                        &ComponentContainer<ComponentType>,
                        &'a ComponentContainer<ComponentType>,
                    >(container)
                }),
            ComponentContainerGuard::Mutable(_guard) => None,
        }
    }

    pub(crate) fn as_mutable<ComponentType: 'static>(
        &mut self,
    ) -> Option<&'a mut ComponentContainer<ComponentType>> {
        match self {
            ComponentContainerGuard::Immutable(_guard) => None,
            ComponentContainerGuard::Mutable(guard) => guard
                .downcast_mut::<ComponentContainer<ComponentType>>()
                .map(|container| unsafe {
                    std::mem::transmute::<
                        &mut ComponentContainer<ComponentType>,
                        &'a mut ComponentContainer<ComponentType>,
                    >(container)
                }),
        }
    }
}

pub struct ComponentViewIterator<'a> {
    object_container_index: Option<ObjectContainerIndex>,
    component_containers: Vec<(std::any::TypeId, ComponentContainerGuard<'a>)>,
}

impl<'a> ComponentViewIterator<'a> {
    pub fn new(component_containers: Vec<(std::any::TypeId, ComponentContainerGuard<'a>)>) -> Self {
        Self {
            object_container_index: None,
            component_containers,
        }
    }

    pub fn with_object_container_index(
        mut self,
        object_container_index: ObjectContainerIndex,
    ) -> Self {
        self.object_container_index = Some(object_container_index);
        self
    }

    pub fn object_index(
        &self,
        object_index_in_object_container: ObjectIndexInObjectContainer,
    ) -> ObjectIndex {
        let object_container_index = self.object_container_index.expect(
            "object container index is missing; this iterator must come from Container::iter_component_view_iters",
        );

        ObjectIndex::new(object_container_index, object_index_in_object_container)
    }

    #[track_caller]
    pub fn component_container<T: 'static>(
        &self,
        index: usize,
    ) -> Option<&'a ComponentContainer<T>> {
        self.component_containers
            .get(index)
            .and_then(|(type_id, container)| {
                if *type_id == std::any::TypeId::of::<T>() {
                    container.as_immutable::<T>()
                } else {
                    None
                }
            })
    }

    #[track_caller]
    pub fn component_container_unchecked<T: 'static>(
        &self,
        index: usize,
    ) -> &'a ComponentContainer<T> {
        match self.component_container::<T>(index) {
            Some(container) => container,
            None => panic!(
                "Component type at index {index} is not of type `{}`",
                std::any::type_name::<T>()
            ),
        }
    }

    #[track_caller]
    pub fn component_container_mut<T: 'static>(
        &mut self,
        index: usize,
    ) -> Option<&'a mut ComponentContainer<T>> {
        self.component_containers
            .get_mut(index)
            .and_then(|(type_id, container)| {
                if *type_id == std::any::TypeId::of::<T>() {
                    container.as_mutable::<T>()
                } else {
                    None
                }
            })
    }

    #[track_caller]
    pub fn component_container_mut_unchecked<T: 'static>(
        &mut self,
        index: usize,
    ) -> &'a mut ComponentContainer<T> {
        match self.component_container_mut::<T>(index) {
            Some(container) => container,
            None => panic!(
                "Component type at index {index} is not of type `mut {}`",
                std::any::type_name::<T>()
            ),
        }
    }
}
