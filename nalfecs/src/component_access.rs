#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ComponentAccess {
    Immutable(std::any::TypeId),
    Mutable(std::any::TypeId),
}

impl ComponentAccess {
    pub fn mutable<T: 'static>() -> Self {
        ComponentAccess::Mutable(std::any::TypeId::of::<T>())
    }

    pub fn immutable<T: 'static>() -> Self {
        ComponentAccess::Immutable(std::any::TypeId::of::<T>())
    }

    pub fn type_id(&self) -> std::any::TypeId {
        match self {
            ComponentAccess::Immutable(type_id) | ComponentAccess::Mutable(type_id) => *type_id,
        }
    }

    pub fn is_mutable(&self) -> bool {
        matches!(self, ComponentAccess::Mutable(_))
    }
}
