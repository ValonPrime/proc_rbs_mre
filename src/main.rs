use bevy::ecs::archetype::Archetypes;
use bevy::ecs::component::Components;
use bevy::ecs::entity::Entities;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

#[derive(Component, Copy, Clone)]
pub struct MarkerParent;
#[derive(Component, Copy, Clone)]
pub struct MarkerChild;

// Enables/Disables the Workaround suggested in the dc
const DO_WORKAROUND: bool = true;

const LINEAR_DAMPING: f32 = 0.05;
const ANGULAR_DAMPING: f32 = 0.05;

const DEFAULT_RB: RigidBody = RigidBody::Dynamic;
const DEFAULT_DAMPING: Damping = Damping {
    linear_damping: LINEAR_DAMPING,
    angular_damping: ANGULAR_DAMPING,
};

const DEFAULT_MASS: ColliderMassProperties = ColliderMassProperties::Density(7.874);
const DEFAULT_RESTITUTION: Restitution = Restitution::coefficient(0.7);
const DEFAULT_FRICTION: Friction = Friction::coefficient(0.0);

fn main() {
    let bevy_default = DefaultPlugins;
    let rapier = RapierPhysicsPlugin::<NoUserData>::default();
    let rapier_debug = RapierDebugRenderPlugin {
        style: DebugRenderStyle {
            collider_dynamic_color: [0.0, 1.0, 0.3, 1.0], // Red
            collider_kinematic_color: [90.0, 1.0, 0.3, 1.0], // Green
            collider_fixed_color: [180.0, 1.0, 0.4, 1.0], // Cyan
            collider_parentless_color: [270.0, 1.0, 0.4, 1.0], // Purple
            ..default()
        },
        ..default()
    };

    App::new()
        .add_plugins((bevy_default, rapier))
        .add_plugins(rapier_debug)
        .add_systems(Startup, (setup_scene, setup_proc, setup_static))
        .add_systems(Update, (convert_proc, inspect, debug_rbs))
        .run();
}

pub fn convert_proc(
    key_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,

    parents: Query<Entity, (With<MarkerParent>, With<Transform>, Without<MarkerChild>)>,
    children: Query<Entity, (With<MarkerChild>, With<Transform>, Without<MarkerParent>)>,
) {
    if key_input.just_pressed(KeyCode::KeyG) {
        println!("Converting");
        let mut parent_amount = 0;
        for parent in &parents {
            parent_amount += 1;
            commands
                .entity(parent)
                .remove::<MarkerParent>()
                .insert((DEFAULT_RB.clone(), DEFAULT_DAMPING.clone()));
        }
        println!("ParentAmount: {}", parent_amount);

        let mut child_amount = 0;
        for child in &children {
            child_amount += 1;
            commands.entity(child).remove::<MarkerChild>().insert((
                DEFAULT_MASS.clone(),
                DEFAULT_RESTITUTION.clone(),
                DEFAULT_FRICTION.clone(),
            ));
        }
        println!("ChildAmount: {}", child_amount);
        println!("Finished");
    }
}

pub fn debug_rbs(
    rbs: Query<(Entity, &Transform, &RigidBody)>,
    cols: Query<(Entity, &Transform, &Collider, &Parent), Without<RigidBody>>,
    mut gizmos: Gizmos,
) {
    for (entity, transform, _) in &rbs {
        gizmos.arrow(transform.translation, transform.translation + (Vec3::Y * 2.0), Color::srgba(1.0, 0.0, 0.0, 1.0));
        println!("Rb: {}, Pos: {}", entity, transform.translation)
    }

    for (entity, _, _, parent) in &cols {
        let result = rbs.get(parent.get());
        match result {
            Ok(_) => println!("Col: {}, Has RB parent: true", entity),
            Err(_) => println!("Col: {}, Has RB parent: false", entity),
        }
    }
}

fn inspect(
    key_input: Res<ButtonInput<KeyCode>>,
    all_entities: Query<Entity>,
    entities: &Entities,
    archetypes: &Archetypes,
    components: &Components,
) {
    if key_input.just_pressed(KeyCode::F1) {
        for entity in all_entities.iter() {
            println!("Entity: {:?}", entity);
            if let Some(entity_location) = entities.get(entity) {
                if let Some(archetype) = archetypes.get(entity_location.archetype_id) {
                    for component in archetype.components() {
                        if let Some(info) = components.get_info(component) {
                            println!("\tComponent: {}", info.name());
                        }
                    }
                }
            }
        }
    }
}

fn setup_proc(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Test rb
    let mut parent = commands.spawn((
        Transform::from_xyz(1.0, 2.0, 0.0),
        InheritedVisibility::default(),
        MarkerParent,
    ));

    // Adding a fixed RB and converting that later fixes the problem
    if DO_WORKAROUND {
        parent.insert(RigidBody::Fixed);
    }

    parent.with_children(|child_builder| {
        child_builder.spawn((
            Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
            MeshMaterial3d(materials.add(Color::srgb(1.0, 0.3, 0.3))),
            Transform::from_xyz(0.0, 0.0, 0.0),
            Collider::cuboid(0.5, 0.5, 0.5),
            MarkerChild,
        ));
    });
}

fn setup_static(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Test rb
    let mut rb = commands.spawn((
        Transform::from_xyz(-1.0, 2.0, 0.0),
        InheritedVisibility::default(),
    ));

    rb.insert(RigidBody::Dynamic);
    rb.insert(Damping {
        linear_damping: 0.05,
        angular_damping: 0.05,
    });

    rb.with_children(|child_builder| {
        let mut child = child_builder.spawn((
            Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
            MeshMaterial3d(materials.add(Color::srgb(0.3, 1.0, 0.3))),
            Transform::from_xyz(0.0, 0.0, 0.0),
            Collider::cuboid(0.5, 0.5, 0.5),
        ));

        child.insert(Restitution::coefficient(0.7));
        child.insert(Friction::coefficient(0.0));
        child.insert(ColliderMassProperties::Density(7.874));
    });
}

fn setup_scene(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 15.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
