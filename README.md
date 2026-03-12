# Todo
* consider using small buffer optimization in iterators
* create a `type_descriptors!` macro
  ```rust
  let view_desc = container.view_descriptor(&[
        nalfecs::ComponentAccess::immutable::<component::Transform>(),
        nalfecs::ComponentAccess::mutable::<component::RigidBody>(),
        nalfecs::ComponentAccess::mutable::<component::Appearance>(),
    ]);
  ```
  ```rust
  let view_desc = container.view_descriptor(
    nalfecs::type_descriptors!(<
      component::Transform,
      mut component::RigidBody,
      mut component::Appearance,
    >)
  );
  ```

* consider generating `Container` also
  * remove `GenerationalSlotMap<Arc<dyn ContainerType>>` from it and generate fields for each 
  * this may eliminate the possibility of dynamic container adding and removing

* choose archetype or object

* be able to work on structs with unnamed fields
* support generics
