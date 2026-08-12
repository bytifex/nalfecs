use std::any::Any;

use crate::{
    ComponentAccess, ComponentViewDescriptorForObjectContainer, ComponentViewIterator, Object,
    ObjectIndexInObjectContainer,
};

pub trait AsAny {
    fn as_any(&self) -> &dyn Any;
}

impl<T> AsAny for T
where
    T: ObjectContainer + Any,
{
    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub trait ObjectContainer: Any + AsAny + Send + Sync {
    fn view_descriptor(
        &self,
        component_accesses: &[ComponentAccess],
    ) -> Option<ComponentViewDescriptorForObjectContainer>;

    fn iter_views_for(
        &self,
        desc: &ComponentViewDescriptorForObjectContainer,
    ) -> Option<ComponentViewIterator<'_>>;
}

pub trait ObjectContainerFor<T>: ObjectContainer
where
    T: Object<Container = Self>,
    Self: Sized,
{
    fn add_object(&self, object: T) -> ObjectIndexInObjectContainer;
    fn remove_object(&self, index: ObjectIndexInObjectContainer) -> Option<T>;
}
