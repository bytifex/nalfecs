use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};

use rand::{RngExt, SeedableRng};

enum ComponentToModify {
    RigidBody,
    Appearance,
}

mod component {
    #[derive(Debug)]
    pub struct Transform(#[allow(dead_code)] pub String);
    #[derive(Debug)]
    pub struct RigidBody(#[allow(dead_code)] pub String);
    #[derive(Debug)]
    pub struct Appearance(#[allow(dead_code)] pub String);
}

#[test]
fn compilation() {
    #[nalfecs::object(container_name = "RigidBoxContainer")]
    struct RigidBox {
        label: String,
        transform: component::Transform,
        rigid_body: component::RigidBody,
        appearance: component::Appearance,
    }

    let rigid_box_container = RigidBoxContainer::new();

    const ITEM_COUNT: usize = 10;
    for i in 0..ITEM_COUNT {
        rigid_box_container.add(RigidBox {
            label: format!("rigid_box_{i}"),
            transform: component::Transform(format!("transform_{i}")),
            rigid_body: component::RigidBody(format!("rigid_body_{i}")),
            appearance: component::Appearance(format!("appearance_{i}")),
        });
    }

    let container = nalfecs::Container::new([
        Box::new(rigid_box_container) as Box<dyn nalfecs::ObjectContainer>
    ]);

    let view_desc = container.view_descriptor(&[
        nalfecs::ComponentAccess::immutable::<component::Transform>(),
        nalfecs::ComponentAccess::mutable::<component::RigidBody>(),
        nalfecs::ComponentAccess::mutable::<component::Appearance>(),
    ]);

    let iter = nalfecs::container_iter!(
        <component::Transform, mut component::RigidBody, mut component::Appearance>,
        container,
        &view_desc,
    );

    // {
    //     let view_desc = &view_desc;
    //     assert_eq!(
    //         view_desc.number_of_components(),
    //         3,
    //         "number of components do not match with view descriptor",
    //     );
    //     container
    //         .iter_object_container_view_iters(view_desc)
    //         .map(|mut iter| {
    //             let container_0 = iter.component_container_unchecked::<component::Transform>(0);
    //             let container_1 = iter.component_container_mut_unchecked::<component::RigidBody>(1);
    //             let container_2 =
    //                 iter.component_container_mut_unchecked::<component::Appearance>(2);
    //             assert_eq!(
    //                 container_0.len(),
    //                 container_1.len(),
    //                 "component length mismatch, component indices = (0, 1)"
    //             );
    //             assert_eq!(
    //                 container_0.len(),
    //                 container_2.len(),
    //                 "component length mismatch, component indices = (0, 2)"
    //             );
    //             container_0
    //                 .iter_with_index()
    //                 .zip(container_1.iter_mut().zip(container_2.iter_mut()))
    //                 .map(move |(component_0, (component_1, component_2))| {
    //                     let (object_index_in_object_container, component_0) = component_0;
    //                     let object_index = iter.object_index(object_index_in_object_container);
    //                     (object_index, component_0, component_1, component_2)
    //                 })
    //         })
    //         .flatten()
    // }

    let item_count = operate_on(iter);
    assert_eq!(item_count, ITEM_COUNT);
}

fn operate_on<'a>(
    rigid_bodies: impl Iterator<
        Item = (
            nalfecs::ObjectIndex,
            &'a component::Transform,
            &'a mut component::RigidBody,
            &'a mut component::Appearance,
        ),
    >,
) -> usize {
    let mut count = 0;
    for (_object_index, _transform, _rigid_body, _appearance) in rigid_bodies {
        // println!("{:?}, {:?}, {:?}", _transform, _rigid_body, _appearance);
        count += 1;
    }
    count
}

#[test]
fn add_remove_through_container() {
    #[nalfecs::object(container_name = "RigidBoxContainer")]
    struct RigidBox {
        label: String,
        transform: component::Transform,
        rigid_body: component::RigidBody,
        appearance: component::Appearance,
    }

    let container = nalfecs::Container::new([
        Box::new(RigidBoxContainer::new()) as Box<dyn nalfecs::ObjectContainer>
    ]);

    let index = container
        .add(RigidBox {
            label: "rigid_box_0".to_string(),
            transform: component::Transform("transform_0".to_string()),
            rigid_body: component::RigidBody("rigid_body_0".to_string()),
            appearance: component::Appearance("appearance_0".to_string()),
        })
        .expect("expected matching archetype container");

    let removed = container
        .remove::<RigidBox>(index)
        .expect("expected to remove added object");

    assert_eq!(removed.label, "rigid_box_0");
    assert_eq!(removed.transform.0, "transform_0");
    assert_eq!(removed.rigid_body.0, "rigid_body_0");
    assert_eq!(removed.appearance.0, "appearance_0");

    assert!(container.remove::<RigidBox>(index).is_none());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn parallel_add_remove_through_container() {
    #[nalfecs::object(container_name = "RigidBoxContainer")]
    struct RigidBox {
        label: String,
        transform: component::Transform,
        rigid_body: component::RigidBody,
        appearance: component::Appearance,
    }

    #[nalfecs::object(container_name = "RigidSphereContainer")]
    struct RigidSphere {
        label: String,
        transform: component::Transform,
        rigid_body: component::RigidBody,
        appearance: component::Appearance,
    }

    const TASK_COUNT: usize = 8;
    const ITEMS_PER_TASK: usize = 100;
    const TOTAL_ITEMS_PER_TYPE: usize = TASK_COUNT * ITEMS_PER_TASK;
    const TOTAL_ITEMS: usize = TOTAL_ITEMS_PER_TYPE * 2;
    const MARKER_ONE: &str = "|m1|";
    const MARKER_TWO: &str = "|m2|";

    let container = Arc::new(nalfecs::Container::new([
        Box::new(RigidBoxContainer::new()) as Box<dyn nalfecs::ObjectContainer>,
        Box::new(RigidSphereContainer::new()) as Box<dyn nalfecs::ObjectContainer>,
    ]));

    let rigid_box_indices = Arc::new(Mutex::new(Vec::with_capacity(TOTAL_ITEMS_PER_TYPE)));
    let rigid_sphere_indices = Arc::new(Mutex::new(Vec::with_capacity(TOTAL_ITEMS_PER_TYPE)));

    let add_tasks: Vec<_> = (0..TASK_COUNT)
        .map(|task_id| {
            let container = Arc::clone(&container);
            let rigid_box_indices = Arc::clone(&rigid_box_indices);
            let rigid_sphere_indices = Arc::clone(&rigid_sphere_indices);

            tokio::spawn(async move {
                // Seed deterministically per task to avoid relying on OS entropy APIs.
                let mut rng = rand::rngs::SmallRng::seed_from_u64(task_id as u64 + 1);

                for item_id in 0..ITEMS_PER_TASK {
                    let add_box = || {
                        container
                            .add(RigidBox {
                                label: format!("rigid_box_{task_id}_{item_id}"),
                                transform: component::Transform(format!(
                                    "transform_{task_id}_{item_id}"
                                )),
                                rigid_body: component::RigidBody(format!(
                                    "rigid_body_{task_id}_{item_id}"
                                )),
                                appearance: component::Appearance(format!(
                                    "appearance_{task_id}_{item_id}"
                                )),
                            })
                            .expect("expected matching object container")
                    };
                    let add_sphere = || {
                        container
                            .add(RigidSphere {
                                label: format!("rigid_sphere_{task_id}_{item_id}"),
                                transform: component::Transform(format!(
                                    "transform_{task_id}_{item_id}"
                                )),
                                rigid_body: component::RigidBody(format!(
                                    "rigid_body_{task_id}_{item_id}"
                                )),
                                appearance: component::Appearance(format!(
                                    "appearance_{task_id}_{item_id}"
                                )),
                            })
                            .expect("expected matching object container")
                    };

                    let (rigid_box_index, rigid_sphere_index) = if rng.random_bool(0.5) {
                        let rigid_box_index = add_box();
                        let rigid_sphere_index = add_sphere();
                        (rigid_box_index, rigid_sphere_index)
                    } else {
                        let rigid_sphere_index = add_sphere();
                        let rigid_box_index = add_box();
                        (rigid_box_index, rigid_sphere_index)
                    };

                    rigid_box_indices
                        .lock()
                        .expect("mutex poisoned")
                        .push(rigid_box_index);
                    rigid_sphere_indices
                        .lock()
                        .expect("mutex poisoned")
                        .push(rigid_sphere_index);
                }
            })
        })
        .collect();

    for join_handle in add_tasks {
        join_handle.await.expect("add task panicked");
    }

    let modifier_one = tokio::spawn(modify_some_components(
        Arc::clone(&container),
        ComponentToModify::RigidBody,
        MARKER_ONE,
    ));
    let modifier_two = tokio::spawn(modify_some_components(
        Arc::clone(&container),
        ComponentToModify::Appearance,
        MARKER_TWO,
    ));

    let modifier_one_count = modifier_one.await.expect("modifier one task panicked");
    let modifier_two_count = modifier_two.await.expect("modifier two task panicked");

    assert_eq!(modifier_one_count, TOTAL_ITEMS);
    assert_eq!(modifier_two_count, TOTAL_ITEMS);

    let rigid_box_indices = Arc::new(Mutex::new(std::mem::take(
        &mut *rigid_box_indices.lock().expect("mutex poisoned"),
    )));
    let rigid_sphere_indices = Arc::new(Mutex::new(std::mem::take(
        &mut *rigid_sphere_indices.lock().expect("mutex poisoned"),
    )));
    let removed_rigid_box_count = Arc::new(AtomicUsize::new(0));
    let removed_rigid_sphere_count = Arc::new(AtomicUsize::new(0));

    let remove_tasks: Vec<_> = (0..TASK_COUNT)
        .map(|_| {
            let container = Arc::clone(&container);
            let rigid_box_indices = Arc::clone(&rigid_box_indices);
            let rigid_sphere_indices = Arc::clone(&rigid_sphere_indices);
            let removed_rigid_box_count = Arc::clone(&removed_rigid_box_count);
            let removed_rigid_sphere_count = Arc::clone(&removed_rigid_sphere_count);

            tokio::spawn(async move {
                loop {
                    let removed_any = {
                        let rigid_box_index_opt =
                            rigid_box_indices.lock().expect("mutex poisoned").pop();
                        if let Some(index) = rigid_box_index_opt {
                            let removed = container
                                .remove::<RigidBox>(index)
                                .expect("expected to remove inserted rigid box");
                            assert!(removed.label.starts_with("rigid_box_"));
                            assert!(removed.rigid_body.0.contains(MARKER_ONE));
                            assert!(removed.appearance.0.contains(MARKER_TWO));
                            removed_rigid_box_count.fetch_add(1, Ordering::Relaxed);
                            true
                        } else {
                            false
                        }
                    };

                    let removed_any = {
                        let rigid_sphere_index_opt =
                            rigid_sphere_indices.lock().expect("mutex poisoned").pop();
                        if let Some(index) = rigid_sphere_index_opt {
                            let removed = container
                                .remove::<RigidSphere>(index)
                                .expect("expected to remove inserted rigid sphere");
                            assert!(removed.label.starts_with("rigid_sphere_"));
                            assert!(removed.rigid_body.0.contains(MARKER_ONE));
                            assert!(removed.appearance.0.contains(MARKER_TWO));
                            removed_rigid_sphere_count.fetch_add(1, Ordering::Relaxed);
                            true
                        } else {
                            removed_any
                        }
                    };

                    if !removed_any {
                        break;
                    }
                }
            })
        })
        .collect();

    for join_handle in remove_tasks {
        join_handle.await.expect("remove task panicked");
    }

    assert_eq!(
        removed_rigid_box_count.load(Ordering::Relaxed),
        TOTAL_ITEMS_PER_TYPE
    );
    assert_eq!(
        removed_rigid_sphere_count.load(Ordering::Relaxed),
        TOTAL_ITEMS_PER_TYPE
    );
}

async fn modify_some_components(
    container: Arc<nalfecs::Container>,
    component_to_modify: ComponentToModify,
    marker: &str,
) -> usize {
    match component_to_modify {
        ComponentToModify::RigidBody => {
            let view_desc = container.view_descriptor(&[
                nalfecs::ComponentAccess::immutable::<component::Transform>(),
                nalfecs::ComponentAccess::mutable::<component::RigidBody>(),
            ]);

            let iter = nalfecs::container_iter!(
                <component::Transform, mut component::RigidBody>,
                container,
                &view_desc,
            );

            let mut modified_count = 0;
            for (_object_index, _transform, rigid_body) in iter {
                rigid_body.0.push_str(marker);
                modified_count += 1;
            }

            modified_count
        }
        ComponentToModify::Appearance => {
            let view_desc = container.view_descriptor(&[
                nalfecs::ComponentAccess::immutable::<component::Transform>(),
                nalfecs::ComponentAccess::mutable::<component::Appearance>(),
            ]);

            let iter = nalfecs::container_iter!(
                <component::Transform, mut component::Appearance>,
                container,
                &view_desc,
            );

            let mut modified_count = 0;
            for (_object_index, _transform, appearance) in iter {
                appearance.0.push_str(marker);
                modified_count += 1;
            }

            modified_count
        }
    }
}

#[test]
fn duplicate_component_type_is_rejected() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/ui/duplicate_component_type.rs");
    tests.compile_fail("tests/ui/duplicate_component_type_alias.rs");
}

#[test]
fn single_component_immutable_iteration() {
    #[nalfecs::object(container_name = "RigidBoxContainer")]
    struct RigidBox {
        label: String,
        transform: component::Transform,
        rigid_body: component::RigidBody,
        appearance: component::Appearance,
    }

    let rigid_box_container = RigidBoxContainer::new();

    const ITEM_COUNT: usize = 5;
    for i in 0..ITEM_COUNT {
        rigid_box_container.add(RigidBox {
            label: format!("rigid_box_{i}"),
            transform: component::Transform(format!("transform_{i}")),
            rigid_body: component::RigidBody(format!("rigid_body_{i}")),
            appearance: component::Appearance(format!("appearance_{i}")),
        });
    }

    let container = nalfecs::Container::new([
        Box::new(rigid_box_container) as Box<dyn nalfecs::ObjectContainer>
    ]);

    // Use component_iter to iterate over just Transform components
    let count = container.component_iter::<component::Transform>().count();
    assert_eq!(count, ITEM_COUNT);
}

#[test]
fn single_component_mutable_iteration() {
    #[nalfecs::object(container_name = "RigidBoxContainer")]
    struct RigidBox {
        label: String,
        transform: component::Transform,
        rigid_body: component::RigidBody,
        appearance: component::Appearance,
    }

    let rigid_box_container = RigidBoxContainer::new();

    const ITEM_COUNT: usize = 5;
    for i in 0..ITEM_COUNT {
        rigid_box_container.add(RigidBox {
            label: format!("rigid_box_{i}"),
            transform: component::Transform(format!("transform_{i}")),
            rigid_body: component::RigidBody(format!("rigid_body_{i}")),
            appearance: component::Appearance(format!("appearance_{i}")),
        });
    }

    let container = nalfecs::Container::new([
        Box::new(rigid_box_container) as Box<dyn nalfecs::ObjectContainer>
    ]);

    // Use component_iter_mut to iterate over mutable RigidBody components
    let mut modified_count = 0;
    for rigid_body in container.component_iter_mut::<component::RigidBody>() {
        rigid_body.0.push_str("|modified|");
        modified_count += 1;
    }

    assert_eq!(modified_count, ITEM_COUNT);
}

#[test]
fn iterating_component_set_exposes_object_index() {
    #[nalfecs::object(container_name = "RigidBoxContainer")]
    struct RigidBox {
        label: String,
        transform: component::Transform,
        rigid_body: component::RigidBody,
        appearance: component::Appearance,
    }

    let rigid_box_container = RigidBoxContainer::new();

    const ITEM_COUNT: usize = 4;
    for i in 0..ITEM_COUNT {
        rigid_box_container.add(RigidBox {
            label: format!("rigid_box_{i}"),
            transform: component::Transform(format!("transform_{i}")),
            rigid_body: component::RigidBody(format!("rigid_body_{i}")),
            appearance: component::Appearance(format!("appearance_{i}")),
        });
    }

    let container = nalfecs::Container::new([
        Box::new(rigid_box_container) as Box<dyn nalfecs::ObjectContainer>
    ]);

    let view_desc = container.view_descriptor(&[
        nalfecs::ComponentAccess::immutable::<component::Transform>(),
        nalfecs::ComponentAccess::mutable::<component::RigidBody>(),
    ]);

    let iter = nalfecs::container_iter!(
        <component::Transform, mut component::RigidBody>,
        container,
        &view_desc,
    );

    let indices: Vec<nalfecs::ObjectIndex> = iter
        .map(|(object_index, _transform, rigid_body)| {
            rigid_body.0.push_str("|seen|");
            object_index
        })
        .collect();

    assert_eq!(indices.len(), ITEM_COUNT);

    let mut removed = 0;
    for index in indices {
        let object = container
            .remove::<RigidBox>(index)
            .expect("expected object reachable by iterated object index");
        assert!(object.rigid_body.0.contains("|seen|"));
        removed += 1;
    }

    assert_eq!(removed, ITEM_COUNT);
}

// todo!()
// #[tokio::test(flavor = "multi_thread")]
// async fn non_send_non_sync() {
//     #[allow(dead_code)]
//     #[nalfecs::object(container_name = "ObjectContainer")]
//     struct Object {
//         component: std::rc::Rc<()>,
//     }
// }
