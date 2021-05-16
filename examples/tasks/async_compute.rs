use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use std::time::{Instant, Duration};
use rand::Rng;
use std::option::Option::Some;
use futures_lite::future;

/// This example shows how to use the ecs system and the AsyncComputeTaskPool
/// to spawn, poll, and complete tasks across systems and system ticks.

// Number of cubes to spawn across the x, y, and z axis
const NUM_CUBES: i32 = 6;

// Used to tag our new entity spawned with tasks
struct Marker;

/// This system generates tasks simulating computationally intensive
/// work that potentially spawns multiple frames/ticks. A separate
/// system, handle_tasks, will poll the spawned tasks on subsequent
/// frames/ticks, and use the results to spawn cubes
fn spawn_tasks(
    mut commands: Commands,
    thread_pool: Res<AsyncComputeTaskPool>,
) {
    for x in 0 ..NUM_CUBES {
        for y in 0 ..NUM_CUBES {
            for z in 0 ..NUM_CUBES {

                // Spawn new task on the AsyncComputeTaskPool
                let task = thread_pool.spawn(async move {

                    let mut rng = rand::thread_rng();
                    let start_time = Instant::now();
                    let duration = Duration::from_secs_f32(rng.gen_range(0.05..0.2));
                    while Instant::now() - start_time < duration {
                        // Simulating doing hard compute work generating translation coords!
                    }

                    // Such hard work, all done!
                    eprintln!("Done generating translation coords x: {} y: {} z: {}", x, y, z);
                    Transform::from_translation(Vec3::new(x as f32, y as f32, z as f32))
                });

                // Spawn new entity, tag it with Marker as a component,
                // and add our new task as a component
                commands.spawn()
                    .insert(Marker)
                    .insert(task);
            }
        }
    }
}

/// This system queries for entities that have both our Marker component
/// as well as a Task<Transform> component. It polls the tasks to see if they're
/// complete. If the task is complete it takes the result, adds a new PbrBundle of components to the
/// entity using the result from the task's work, and removes the task component from the entity.
fn handle_tasks(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut our_entity_tasks: Query<(Entity, &mut Task<Transform>), With<Marker>>
) {
    for (entity, mut task) in our_entity_tasks.iter_mut() {

        if let Some(transform) = future::block_on(future::poll_once(&mut *task)) {

            // Normally we would add our mesh and material assets once
            // and store the handle, but that's for another example
            let box_mesh = meshes.add(Mesh::from(shape::Cube { size: 0.25 }));
            let box_material = materials.add(Color::rgb(1.0, 0.2, 0.3).into());

            // Add our new PbrBundle of components to our tagged entity
            commands.entity(entity).insert_bundle(PbrBundle {
                mesh: box_mesh.clone(),
                material: box_material.clone(),
                transform,
                ..Default::default()
            });

            // Task is complete, so remove task component from entity
            commands.entity(entity).remove::<Task<Transform>>();
        }
    }
}

fn main() {
    App::build()
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup_env.system())
        .add_startup_system(spawn_tasks.system())
        .add_system(handle_tasks.system())
        .run();
}

/// This system IS NOT part of the example, it's only
/// used to setup the light and camera for the environment
fn setup_env(mut commands: Commands) {

    // Used to center camera on spawned cubes
    let offset = || {
        if NUM_CUBES % 2 == 0 {
            (NUM_CUBES / 2) as f32 - 0.5
        } else {
            (NUM_CUBES / 2) as f32
        }
    };

    // lights
    commands.spawn_bundle(PointLightBundle {
        transform: Transform::from_translation(Vec3::new(4.0, 12.0, 15.0)),
        ..Default::default()
    });

    // camera
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_translation(Vec3::new(offset(), offset(), 15.0))
            .looking_at(Vec3::new(offset(), offset(), 0.0), Vec3::Y),
        ..Default::default()
    });
}