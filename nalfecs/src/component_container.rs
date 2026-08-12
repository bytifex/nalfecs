use crate::{GenerationalSlotMap, GenerationalSlotMapIndex, ObjectIndexInObjectContainer};

/// Basically this is the same as `ObjectIndexInObjectContainer`. The underlying
/// data should have the same value for every components in the same object.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ComponentIndex(pub(crate) GenerationalSlotMapIndex);

pub struct ComponentContainer<T> {
    inner: GenerationalSlotMap<T>,
}

impl<T> Default for ComponentContainer<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> ComponentContainer<T> {
    pub fn new() -> Self {
        Self {
            inner: Default::default(),
        }
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn add(&mut self, item: T) -> ComponentIndex {
        ComponentIndex(self.inner.create_object(item))
    }

    pub fn remove(&mut self, index: ComponentIndex) -> Option<T> {
        self.inner.release_object(index.0)
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.inner.iter().map(|(_index, component)| component)
    }

    pub fn iter_with_index(&self) -> impl Iterator<Item = (ObjectIndexInObjectContainer, &T)> {
        self.inner
            .iter()
            .map(|(index, component)| (ObjectIndexInObjectContainer(index), component))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.inner.iter_mut().map(|(_index, component)| component)
    }

    pub fn iter_mut_with_index(
        &mut self,
    ) -> impl Iterator<Item = (ObjectIndexInObjectContainer, &mut T)> {
        self.inner
            .iter_mut()
            .map(|(index, component)| (ObjectIndexInObjectContainer(index), component))
    }

    pub fn get(&self, index: ComponentIndex) -> Option<&T> {
        self.inner.get_ref(index.0)
    }

    pub fn get_mut(&mut self, index: ComponentIndex) -> Option<&mut T> {
        self.inner.get_mut(index.0)
    }
}
