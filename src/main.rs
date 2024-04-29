use bevy::{
    app::{App, FixedPostUpdate, FixedUpdate, PostUpdate, Startup, Update},
    asset::Assets,
    core_pipeline::core_3d::Camera3dBundle,
    ecs::{
        query::With,
        schedule::IntoSystemConfigs,
        system::{Commands, Query, Res, ResMut},
    },
    gizmos::gizmos::Gizmos,
    hierarchy::BuildChildren,
    input::{keyboard::KeyCode, ButtonInput},
    math::{
        primitives::{Capsule3d, Cuboid},
        Quat, Vec3,
    },
    pbr::{
        light_consts, AlphaMode, AmbientLight, CascadeShadowConfigBuilder, DirectionalLight,
        DirectionalLightBundle, PbrBundle, StandardMaterial,
    },
    render::{color::Color, mesh::Mesh, view::InheritedVisibility},
    transform::{
        components::{GlobalTransform, Transform},
        TransformBundle, TransformSystem,
    },
    utils::default,
    DefaultPlugins,
};
use bevy_rapier3d::{
    dynamics::{Ccd, Damping, GravityScale, LockedAxes, RigidBody, Sleeping, Velocity},
    geometry::{Collider, CollisionGroups, Friction, Group},
    plugin::{NoUserData, RapierPhysicsPlugin},
    render::RapierDebugRenderPlugin,
};
use character::{
    update_character_rigidbody_position_system, update_character_velocity_using_input_system,
    CharacterBodyTagComponent, CharacterBundle, CharacterPlayerInputComponent,
    CharacterRotationFromGlobalToCharacterParametersComponent, CharacterTagComponent,
    CharacterVelocityComponent,
};
use math::{
    CylinderCoordinates3d, CylinderCoordinates3dSmoothDampTransitionVariables,
    SmoothDampTransitionVariables,
};
use player::{
    reset_player_roll_on_mouse_input_system,
    transition_player_camera_current_state_rotation_system,
    transition_player_camera_state_distance_system, transition_player_camera_state_focus_system,
    transition_player_camera_state_height_system, transition_player_camera_state_roll_system,
    update_player_camera_desired_state_coordinates_using_input_system,
    update_player_camera_state_roll_using_input_system,
    update_player_camera_transform_using_state_system, PlayerBundle,
    PlayerCameraCurrentStateComponent, PlayerCameraDesiredStateComponent, PlayerCameraState,
    PlayerCameraTransitionStateVariablesComponent, PlayerTagComponent,
};
use std::f32::consts::PI;

mod character;
mod math;
mod player;

fn update_character_rotation_from_player_to_character_system(
    mut character_query: Query<
        (
            &Transform,
            &mut CharacterRotationFromGlobalToCharacterParametersComponent,
        ),
        With<CharacterTagComponent>,
    >,
    player_query: Query<&Transform, With<PlayerTagComponent>>,
) {
    let mut character = character_query.single_mut();
    let player = player_query.single();
    let camera_up = player.up();
    let character_up = character.0.up();

    character.1.rotation_from_global_to_character_quat =
        Quat::from_rotation_arc(*camera_up, *character_up);
}

fn update_character_movement_player_input_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    player_query: Query<&GlobalTransform, With<PlayerTagComponent>>,
    mut character_query: Query<(&mut CharacterPlayerInputComponent,), With<CharacterTagComponent>>,
) {
    let mut character = character_query.single_mut();
    let player_global_transform = player_query.single();

    let mut local_input = Vec3::ZERO;
    if keyboard_input.pressed(KeyCode::KeyW) {
        local_input.z -= 1.0;
    }

    if keyboard_input.pressed(KeyCode::KeyS) {
        local_input.z += 1.0;
    }

    if keyboard_input.pressed(KeyCode::KeyD) {
        local_input.x += 1.0;
    }

    if keyboard_input.pressed(KeyCode::KeyA) {
        local_input.x -= 1.0;
    }
    character.0.global_input = player_global_transform
        .affine()
        .transform_vector3(local_input)
}

// endregion

// region debug systems

fn draw_character_transform_gizmos_system(
    mut gizmos: Gizmos,
    character_query: Query<
        (
            &Transform,
            &CharacterPlayerInputComponent,
            &CharacterRotationFromGlobalToCharacterParametersComponent,
        ),
        With<CharacterTagComponent>,
    >,
) {
    let character = character_query.single();

    gizmos.arrow(
        character.0.translation,
        character.0.translation + *character.0.right(),
        Color::RED.with_a(0.5),
    );

    gizmos.arrow(
        character.0.translation,
        character.0.translation + *character.0.up(),
        Color::GREEN.with_a(0.5),
    );

    gizmos.arrow(
        character.0.translation,
        character.0.translation + *character.0.forward(),
        Color::BLUE.with_a(0.5),
    );
}

fn draw_character_rotation_from_global_to_character_gizmos_system(
    mut gizmos: Gizmos,
    player: Query<&Transform, With<PlayerTagComponent>>,
    character_query: Query<
        (
            &Transform,
            &CharacterRotationFromGlobalToCharacterParametersComponent,
        ),
        With<CharacterTagComponent>,
    >,
) {
    let player = player.single();
    let character = character_query.single();
    gizmos.arrow(
        character.0.translation,
        character.0.translation
            + Quat::mul_vec3(
                character.1.rotation_from_global_to_character_quat,
                *player.forward(),
            ),
        Color::rgb(0.0, 1.0, 1.0),
    );
    gizmos.arrow(
        character.0.translation,
        character.0.translation
            + Quat::mul_vec3(
                character.1.rotation_from_global_to_character_quat,
                *player.right(),
            ),
        Color::rgb(1.0, 0.0, 1.0),
    );
}

fn draw_character_input_gizmos_system(
    mut gizmos: Gizmos,
    character_query: Query<
        (
            &Transform,
            &CharacterPlayerInputComponent,
            &CharacterRotationFromGlobalToCharacterParametersComponent,
        ),
        With<CharacterTagComponent>,
    >,
) {
    let character = character_query.single();

    gizmos.arrow(
        character.0.translation,
        character.0.translation
            + Quat::mul_vec3(
                character.2.rotation_from_global_to_character_quat,
                character.1.global_input,
            ),
        Color::YELLOW,
    );
}

// endregion

// region startup

fn spawn_props_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // stage, floor
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(25.0, 1.0, 25.0)),
            material: materials.add(Color::WHITE),
            transform: Transform::from_xyz(0.0, -0.5, 0.0),
            ..default()
        },
        Collider::cuboid(12.5, 0.5, 12.5),
        CollisionGroups::new(
            Group::from_bits(0b0010).unwrap(),
            Group::from_bits(0b0100).unwrap(),
        ),
    ));

    // stage, obstacle
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
            material: materials.add(Color::WHITE),
            transform: Transform::from_xyz(5.0, 0.5, 5.0),
            ..default()
        },
        Collider::cuboid(0.5, 0.5, 0.5),
        CollisionGroups::new(
            Group::from_bits(0b0010).unwrap(),
            Group::from_bits(0b0100).unwrap(),
        ),
    ));

    // ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 7.0,
    });

    // directional 'sun' light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: light_consts::lux::OVERCAST_DAY,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-PI / 4.),
            ..default()
        },
        cascade_shadow_config: CascadeShadowConfigBuilder {
            first_cascade_far_bound: 4.0,
            maximum_distance: 10.0,
            ..default()
        }
        .into(),
        ..default()
    });
}

fn spawn_character_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands
        .spawn((
            CharacterBundle {
                tag: CharacterTagComponent,
                global_transform: GlobalTransform::default(),
                transform: Transform::from_xyz(0.0, 0.0, 0.0),
                inherited_visibility: InheritedVisibility::default(),
                rotation_from_player_to_character:
                    CharacterRotationFromGlobalToCharacterParametersComponent {
                        rotation_from_global_to_character_quat: Quat::IDENTITY,
                    },
                player_input: CharacterPlayerInputComponent {
                    global_input: Vec3::ZERO,
                },
                velocity: CharacterVelocityComponent {
                    global_velocity: Vec3::ZERO,
                },
            },
            (
                RigidBody::Dynamic,
                Velocity::zero(),
                GravityScale(0.0),
                Sleeping::disabled(),
                Ccd::enabled(),
                LockedAxes::ROTATION_LOCKED,
                Friction {
                    coefficient: 0.0,
                    ..default()
                },
                Damping {
                    linear_damping: 0.0,
                    angular_damping: 0.0,
                },
            ),
        ))
        .with_children(|parent| {
            parent.spawn(PbrBundle {
                transform: Transform::from_xyz(0.0, 1.0, 0.0),
                mesh: meshes.add(Capsule3d::new(0.5, 1.)), // TOOD use height
                material: materials.add(StandardMaterial {
                    base_color: Color::rgba(0.0, 0.0, 1.0, 0.5),
                    alpha_mode: AlphaMode::Blend,
                    ..default()
                }),
                ..default()
            });
            parent.spawn((
                CharacterBodyTagComponent,
                TransformBundle {
                    local: Transform::from_xyz(0.0, 1.0, 0.0), // TOOD use height
                    ..default()
                },
                Collider::ball(0.5),
                CollisionGroups::new(
                    Group::from_bits(0b0100).unwrap(),
                    Group::from_bits(0b0110).unwrap(),
                ),
            ));
        });
}

fn spawn_player_system(mut commands: Commands) {
    // camera
    commands.spawn((
        PlayerBundle {
            tag: PlayerTagComponent,
            camera_current_state: PlayerCameraCurrentStateComponent {
                camera_state: PlayerCameraState {
                    translation_cylinder_coordinates: CylinderCoordinates3d {
                        distance: 15.0,
                        rotation: 0.0,
                        height: 5.0,
                    },
                    focus: Vec3::new(0.0, 4.0, 0.0),
                    roll: 0.0,
                },
            },
            camera_desired_state: PlayerCameraDesiredStateComponent {
                camera_state: PlayerCameraState {
                    translation_cylinder_coordinates: CylinderCoordinates3d {
                        distance: 15.0,
                        rotation: 0.0,
                        height: 5.0,
                    },
                    focus: Vec3::new(0.0, 4.0, 0.0),
                    roll: 0.0,
                },
            },
            camera_transition_variables: PlayerCameraTransitionStateVariablesComponent {
                transition_cylinder_coordinates:
                    CylinderCoordinates3dSmoothDampTransitionVariables {
                        distance: SmoothDampTransitionVariables { velocity: 0.0 },
                        rotation: SmoothDampTransitionVariables { velocity: 0.0 },
                        height: SmoothDampTransitionVariables { velocity: 0.0 },
                    },
                focus: SmoothDampTransitionVariables {
                    velocity: Vec3::ZERO,
                },
                roll: SmoothDampTransitionVariables { velocity: 0.0 },
            },
        },
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 25., -25.)
                .looking_at(Vec3::new(0., 0., 0.), Vec3::Y),
            ..default()
        },
    ));
}

// endregion

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins);
    app.add_plugins((
        RapierPhysicsPlugin::<NoUserData>::default(),
        RapierDebugRenderPlugin::default(),
    ));

    app.add_systems(Startup, spawn_character_system)
        .add_systems(Startup, spawn_player_system)
        .add_systems(Startup, spawn_props_system)
        .add_systems(
            FixedUpdate,
            update_player_camera_state_roll_using_input_system,
        )
        .add_systems(
            FixedUpdate,
            update_player_camera_desired_state_coordinates_using_input_system,
        )
        .add_systems(
            FixedUpdate,
            update_character_rotation_from_player_to_character_system,
        )
        .add_systems(Update, transition_player_camera_state_distance_system)
        .add_systems(Update, transition_player_camera_state_height_system)
        .add_systems(
            Update,
            transition_player_camera_current_state_rotation_system,
        )
        .add_systems(Update, transition_player_camera_state_roll_system)
        .add_systems(Update, transition_player_camera_state_focus_system)
        .add_systems(Update, reset_player_roll_on_mouse_input_system)
        .add_systems(FixedUpdate, update_character_movement_player_input_system)
        .add_systems(Update, update_player_camera_transform_using_state_system)
        .add_systems(FixedUpdate, update_character_velocity_using_input_system)
        .add_systems(
            FixedPostUpdate,
            update_character_rigidbody_position_system.after(TransformSystem::TransformPropagate),
        )
        .add_systems(
            PostUpdate,
            draw_character_rotation_from_global_to_character_gizmos_system,
        )
        .add_systems(PostUpdate, draw_character_transform_gizmos_system)
        .add_systems(PostUpdate, draw_character_input_gizmos_system)
        .run();
}
