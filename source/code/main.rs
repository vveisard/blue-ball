use bevy::{
    app::{App, FixedPreUpdate, PostUpdate, Startup, Update},
    asset::{AssetServer, Assets, Handle, LoadState, UntypedHandle},
    core_pipeline::core_3d::Camera3dBundle,
    ecs::{
        query::With,
        schedule::{common_conditions::in_state, IntoSystemConfigs, NextState, States, SystemSet},
        system::{Commands, Query, Res, ResMut, Resource},
    },
    gizmos::gizmos::Gizmos,
    gltf::Gltf,
    hierarchy::BuildChildren,
    input::{keyboard::KeyCode, ButtonInput},
    math::{
        primitives::{Capsule3d, Cuboid},
        Quat, Vec2, Vec3,
    },
    pbr::{
        light_consts, AlphaMode, AmbientLight, CascadeShadowConfigBuilder, DirectionalLight,
        DirectionalLightBundle, PbrBundle, StandardMaterial,
    },
    render::{color::Color, mesh::Mesh, view::InheritedVisibility},
    scene::SceneBundle,
    transform::{
        components::{GlobalTransform, Transform},
        TransformBundle,
    },
    utils::default,
    DefaultPlugins,
};
use bevy_rapier3d::{
    dynamics::{Ccd, Damping, GravityScale, LockedAxes, RigidBody, Sleeping, Velocity},
    geometry::{Collider, CollisionGroups, Friction, Group},
    plugin::{NoUserData, PhysicsSet, RapierConfiguration, RapierPhysicsPlugin, TimestepMode},
    render::RapierDebugRenderPlugin,
};
use character::{
    update_character_body_try_jump_while_on_stage_system,
    update_character_body_try_land_while_in_air_system,
    update_character_body_velocity_while_in_air_using_movement_velocity_system,
    update_character_body_velocity_while_on_stage_using_movement_velocity_system,
    update_character_body_while_on_stage_system,
    update_character_horizontal_movement_velocity_system,
    update_character_movement_velocity_while_in_air_phase_system,
    update_character_movement_velocity_while_on_stage_system, CharacterBodyTagComponent,
    CharacterBundle, CharacterFallPhaseMovementParametersComponent,
    CharacterMovementParametersComponent, CharacterMovementVariablesComponent,
    CharacterPlayerInputComponent, CharacterRotationFromGlobalToCharacterParametersComponent,
    CharacterTagComponent,
};
use math::{
    CylinderCoordinates3d, CylinderCoordinates3dSmoothDampTransitionVariables,
    SmoothDampTransitionVariables,
};
use player::{
    draw_player_camera_focus_gizmos_system, reset_player_roll_on_mouse_input_system,
    transition_player_camera_current_state_rotation_system,
    transition_player_camera_state_distance_system, transition_player_camera_state_focus_system,
    transition_player_camera_state_height_system, transition_player_camera_state_roll_system,
    update_player_camera_desired_state_coordinates_using_input_system,
    update_player_camera_state_roll_using_input_system,
    update_player_camera_transform_using_state_system, PlayerBundle,
    PlayerCameraCurrentStateComponent, PlayerCameraDesiredStateComponent, PlayerCameraState,
    PlayerCameraTransitionStateVariablesComponent, PlayerTagComponent,
};
use std::{f32::consts::PI, time::Duration};

mod character;
mod math;
mod player;

/// resource for the next zone
#[derive(Resource)]
struct NextZoneResource {
    /// all asset handle for the zone
    /// nb, includes the main gltf asset
    asset_handles: Vec<UntypedHandle>,

    /// asset handle for the main gltf asset
    main_gltf_asset_handle: Handle<Gltf>,

    /// flag for
    did_spawn_main_gltf: bool,
}

/// state of the app
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
enum AppState {
    #[default]
    None,
    LoadNextZone,
    SetupNextZone,
    Play,
}

/// system to transition [AppState] from [AppState::LoadNextZone] to [AppState::SetupNextZone] when all assets of next zone are loaded.
fn transition_app_state_from_load_next_zone_to_setup_next_zone_when_next_zone_assets_loaded_system(
    asset_server: Res<AssetServer>,
    next_zone: Res<NextZoneResource>,
    mut next_app_state: ResMut<NextState<AppState>>,
) {
    // panics when any asset is unloaded
    let all_loaded = next_zone
        .asset_handles
        .iter()
        .map(|asset_handle| asset_server.get_load_state(asset_handle.id()))
        .all(|f| f.unwrap() == LoadState::Loaded);

    if !all_loaded {
        return;
    }

    // TODO error when not loaded

    next_app_state.set(AppState::SetupNextZone);
}

/// system to transition [AppState] from [AppState::SetupNextZone] to [AppState::Play].
fn transition_app_state_from_setup_next_zone_to_play_when_zone_spawned_system(
    mut commands: Commands,
    next_zone: Res<NextZoneResource>,
    mut next_app_state: ResMut<NextState<AppState>>,
) {
    if !next_zone.did_spawn_main_gltf {
        return;
    }

    commands.remove_resource::<NextZoneResource>();
    next_app_state.set(AppState::Play);
}

/// system to spawn main gltf asset of next zone
fn spawn_scene_using_next_zone_resource_system(
    mut commands: Commands,
    mut zone_loading_resource: ResMut<NextZoneResource>,
    gltf_assets: Res<Assets<Gltf>>,
) {
    if zone_loading_resource.did_spawn_main_gltf {
        return;
    }

    println!("spawn_scene_using_next_zone_resource_system");

    let main_zone_asset_handle = zone_loading_resource.main_gltf_asset_handle.clone();

    // if the GLTF has loaded, we can navigate its contents
    if let Some(gltf) = gltf_assets.get(&main_zone_asset_handle) {
        // spawn the first scene in the file
        commands.spawn(SceneBundle {
            scene: gltf.scenes[0].clone(),
            ..Default::default()
        });

        zone_loading_resource.did_spawn_main_gltf = true;
    } else {
        panic!("asset missing {}", main_zone_asset_handle.id());
    }
}

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

    character.1.rotation_from_camera_to_character_quat =
        Quat::from_rotation_arc(*camera_up, *character_up);
}

fn update_character_movement_player_input_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    player_query: Query<(&Transform, &GlobalTransform), With<PlayerTagComponent>>,
    mut character_query: Query<
        (
            &CharacterRotationFromGlobalToCharacterParametersComponent,
            &mut CharacterPlayerInputComponent,
        ),
        With<CharacterTagComponent>,
    >,
) {
    let mut character = character_query.single_mut();
    let player = player_query.single();

    let player_global_transform = player.1;

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

    // transform input from camera space to global space
    let global_movement_input = player_global_transform
        .affine()
        .transform_vector3(local_input);

    // println!("{}, {}", local_input, global_input);

    character.1.global_movement_player_input = Quat::mul_vec3(
        character.0.rotation_from_camera_to_character_quat,
        global_movement_input,
    );
}

fn update_character_jump_player_input_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut character_query: Query<(&mut CharacterPlayerInputComponent,), With<CharacterTagComponent>>,
) {
    let mut character = character_query.single_mut();

    let mut local_input = false;
    if keyboard_input.just_pressed(KeyCode::Space) {
        local_input = true;
    }
    // println!("{}, {}", local_input, global_input);

    character.0.do_activate_jump_input = local_input
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
        character.0.translation + *character.0.forward(),
        Color::BLUE.with_a(0.5),
    );

    // gizmos.arrow(
    //     character.0.translation,
    //     character.0.translation + *character.0.up(),
    //     Color::GREEN.with_a(0.5),
    // );
}

fn draw_character_rotation_from_global_to_character_gizmos_system(
    mut gizmos: Gizmos,
    player_query: Query<&Transform, With<PlayerTagComponent>>,
    character_query: Query<
        (
            &Transform,
            &CharacterRotationFromGlobalToCharacterParametersComponent,
        ),
        With<CharacterTagComponent>,
    >,
) {
    let player = player_query.single();
    let character = character_query.single();
    gizmos.arrow(
        character.0.translation,
        character.0.translation
            + Quat::mul_vec3(
                character.1.rotation_from_camera_to_character_quat,
                *player.forward(),
            ),
        Color::rgb(0.0, 1.0, 1.0),
    );
    gizmos.arrow(
        character.0.translation,
        character.0.translation
            + Quat::mul_vec3(
                character.1.rotation_from_camera_to_character_quat,
                *player.right(),
            ),
        Color::rgb(1.0, 0.0, 1.0),
    );
}

fn draw_character_input_gizmos_system(
    mut gizmos: Gizmos,
    character_query: Query<
        (&Transform, &CharacterPlayerInputComponent),
        With<CharacterTagComponent>,
    >,
) {
    let character = character_query.single();

    gizmos.arrow(
        character.0.translation,
        character.0.translation + character.1.global_movement_player_input,
        Color::WHITE,
    );
}

fn draw_character_body_velocity_gizmos_system(
    mut gizmos: Gizmos,
    character_query: Query<(&Transform, &Velocity), With<CharacterTagComponent>>,
) {
    let character = character_query.single();

    gizmos.arrow(
        character.0.translation,
        character.0.translation + character.1.linvel,
        Color::YELLOW,
    );
}

fn draw_character_vertical_movement_velocity_gizmos_system(
    mut gizmos: Gizmos,
    character_query: Query<
        (&Transform, &CharacterMovementVariablesComponent),
        With<CharacterTagComponent>,
    >,
) {
    let character = character_query.single();

    let next_body_velocity = character.0.up() * character.1.local_vertical_velocity;

    gizmos.arrow(
        character.0.translation,
        character.0.translation + next_body_velocity,
        Color::ORANGE_RED,
    );
}

fn draw_character_horizontal_movement_velocity_gizmos_system(
    mut gizmos: Gizmos,
    character_query: Query<
        (&Transform, &CharacterMovementVariablesComponent),
        With<CharacterTagComponent>,
    >,
) {
    let character = character_query.single();

    let rotation_from_global_up_to_character_up =
        Quat::from_rotation_arc(Vec3::Y, *character.0.up());

    let next_body_velocity = Quat::mul_vec3(
        rotation_from_global_up_to_character_up,
        Vec3::new(
            character.1.global_horizontal_velocity.x,
            0.0,
            character.1.global_horizontal_velocity.y,
        ),
    );

    gizmos.arrow(
        character.0.translation,
        character.0.translation + next_body_velocity,
        Color::ORANGE_RED,
    );
}

// endregion

// region startup

fn setup_next_zone_to_greenhillzone_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut next_app_state: ResMut<NextState<AppState>>,
) {
    let asset_handle = asset_server.load::<Gltf>("zone/greenhillzone.glb");

    commands.insert_resource(NextZoneResource {
        asset_handles: Vec::from([asset_handle.clone().untyped()]),
        main_gltf_asset_handle: asset_handle,
        did_spawn_main_gltf: false,
    });

    next_app_state.set(AppState::LoadNextZone);
}

fn spawn_props_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // zone, floor
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

    // zone, ramp
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(5.0, 5.0, 5.0)),
            material: materials.add(Color::WHITE),
            transform: Transform::from_xyz(0.0, 0.0, -5.0)
                .with_rotation(Quat::from_rotation_x(-PI / 4.)),
            ..default()
        },
        Collider::cuboid(2.5, 2.5, 2.5),
        CollisionGroups::new(
            Group::from_bits(0b0010).unwrap(),
            Group::from_bits(0b0100).unwrap(),
        ),
    ));

    // zone, obstacle
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
                        rotation_from_camera_to_character_quat: Quat::IDENTITY,
                    },
                player_input: CharacterPlayerInputComponent {
                    global_movement_player_input: Vec3::ZERO,
                    do_activate_jump_input: false,
                },
                fall_phase_movement_parameters: CharacterFallPhaseMovementParametersComponent {
                    maximum_down_speed: 20.0,
                    maximum_up_speed: 25.0,
                    down_acceleration: 0.4,
                },
                movement_variables: CharacterMovementVariablesComponent {
                    global_horizontal_velocity: Vec2::ZERO,
                    local_vertical_velocity: 0.0,
                },
                movement_parameters: CharacterMovementParametersComponent {
                    global_horizontal_acceleration: 0.4,
                    global_horizontal_drag: 0.2,
                },
            },
            (
                RigidBody::Dynamic,
                Velocity::zero(),
                GravityScale(0.0),
                Sleeping::disabled(),
                Ccd::enabled(),
                LockedAxes::ROTATION_LOCKED,
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
                Collider::ball(0.25),
                CollisionGroups::new(
                    Group::from_bits(0b0100).unwrap(),
                    Group::from_bits(0b0110).unwrap(),
                ),
                Friction {
                    coefficient: 0.0,
                    combine_rule: bevy_rapier3d::dynamics::CoefficientCombineRule::Min,
                },
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
                    local_cylinder_coordinates: CylinderCoordinates3d {
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
                    local_cylinder_coordinates: CylinderCoordinates3d {
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

/// system set for character movement velocity
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct CharacterPhaseMovementVelocitySystemSet;

const DEFAULT_TIMESTEP: Duration = Duration::from_micros(15625);

fn main() {
    let mut app = App::new();

    app.insert_state(AppState::None);
    app.insert_resource(RapierConfiguration {
        timestep_mode: TimestepMode::Fixed {
            dt: DEFAULT_TIMESTEP.as_secs_f32(),
            substeps: 1,
        },
        ..default()
    });

    app.add_plugins(DefaultPlugins);
    app.add_plugins((
        RapierPhysicsPlugin::<NoUserData>::default().in_fixed_schedule(),
        RapierDebugRenderPlugin::default(),
    ));

    app.add_systems(Startup, spawn_character_system)
        .add_systems(Startup, spawn_player_system)
        .add_systems(Startup, spawn_props_system)
        .add_systems(Startup, setup_next_zone_to_greenhillzone_system);

    app.add_systems(Update, transition_app_state_from_load_next_zone_to_setup_next_zone_when_next_zone_assets_loaded_system.run_if(in_state(AppState::LoadNextZone)));
    app.add_systems(
        Update,
        transition_app_state_from_setup_next_zone_to_play_when_zone_spawned_system
            .run_if(in_state(AppState::SetupNextZone)),
    );

    app.add_systems(
        Update,
        spawn_scene_using_next_zone_resource_system.run_if(in_state(AppState::SetupNextZone)),
    );

    app.add_systems(
        FixedPreUpdate,
        update_character_rotation_from_player_to_character_system,
    )
    .add_systems(
        FixedPreUpdate,
        (
            update_character_horizontal_movement_velocity_system,
            update_character_movement_velocity_while_on_stage_system,
            update_character_movement_velocity_while_in_air_phase_system,
        )
            .in_set(CharacterPhaseMovementVelocitySystemSet),
    );

    // TODO explicit system set
    app.add_systems(
        FixedPreUpdate,
        (
            update_character_body_try_jump_while_on_stage_system,
            update_character_body_velocity_while_on_stage_using_movement_velocity_system,
            update_character_body_velocity_while_in_air_using_movement_velocity_system,
        )
            .chain()
            .after(CharacterPhaseMovementVelocitySystemSet),
    );

    app.add_systems(
        FixedPreUpdate,
        (
            update_character_body_while_on_stage_system,
            update_character_body_try_land_while_in_air_system,
        )
            .before(PhysicsSet::StepSimulation),
    );

    app.add_systems(Update, update_player_camera_state_roll_using_input_system)
        .add_systems(
            Update,
            update_player_camera_desired_state_coordinates_using_input_system,
        )
        .add_systems(Update, update_character_movement_player_input_system)
        .add_systems(Update, update_character_jump_player_input_system)
        .add_systems(Update, transition_player_camera_state_distance_system)
        .add_systems(Update, transition_player_camera_state_height_system)
        .add_systems(
            Update,
            transition_player_camera_current_state_rotation_system,
        )
        .add_systems(Update, transition_player_camera_state_roll_system)
        .add_systems(Update, transition_player_camera_state_focus_system)
        .add_systems(Update, reset_player_roll_on_mouse_input_system)
        .add_systems(Update, update_player_camera_transform_using_state_system);

    app.add_systems(
        PostUpdate,
        draw_character_rotation_from_global_to_character_gizmos_system,
    )
    .add_systems(PostUpdate, draw_character_transform_gizmos_system)
    .add_systems(PostUpdate, draw_character_input_gizmos_system)
    .add_systems(PostUpdate, draw_player_camera_focus_gizmos_system)
    .add_systems(PostUpdate, draw_character_body_velocity_gizmos_system)
    .add_systems(
        PostUpdate,
        draw_character_horizontal_movement_velocity_gizmos_system,
    )
    .add_systems(
        PostUpdate,
        draw_character_vertical_movement_velocity_gizmos_system,
    );

    app.run();
}
