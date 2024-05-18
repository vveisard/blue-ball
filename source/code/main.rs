use bevy::{
    app::{App, FixedPreUpdate, FixedUpdate, PostUpdate, Startup, Update}, asset::{AssetServer, Assets, Handle, LoadState, UntypedHandle}, core_pipeline::core_3d::Camera3dBundle, ecs::{
        entity::Entity,
        query::With,
        schedule::{
            common_conditions::in_state, IntoSystemConfigs, NextState, OnEnter, States, SystemSet,
        },
        system::{Commands, Query, Res, ResMut, Resource},
    }, gizmos::gizmos::Gizmos, gltf::{Gltf, GltfMesh}, hierarchy::BuildChildren, input::{keyboard::KeyCode, ButtonInput}, math::{
        primitives::{Capsule3d, Cuboid},
        Affine3A, Quat, Vec2, Vec3,
    }, pbr::{
        light_consts, AlphaMode, AmbientLight, CascadeShadowConfigBuilder, DirectionalLight,
        DirectionalLightBundle, PbrBundle, StandardMaterial,
    }, render::{color::Color, mesh::Mesh, view::InheritedVisibility}, scene::SceneBundle, transform::{
        components::{GlobalTransform, Transform},
        TransformBundle,
    }, utils::default, DefaultPlugins
};
use bevy_rapier3d::{
    dynamics::{Ccd, Damping, GravityScale, LockedAxes, RigidBody, Sleeping, Velocity},
    geometry::{Collider, CollisionGroups, ComputedColliderShape, Friction, Group},
    plugin::{NoUserData, PhysicsSet, RapierConfiguration, RapierPhysicsPlugin, TimestepMode},
    render::RapierDebugRenderPlugin,
};
use cylinder_camera::{
    transition_desired_transform_to_transform_system, apply_desired_transform_using_cylinder_coordinates_system, apply_lookat_to_transform_system, set_cylinder_coordinates_for_desired_transform_translation_using_input_system, set_desired_transform_rotation_to_observed_entity_local_up_behavior_system, set_desired_transform_translation_to_observed_entiy_transform_translation_behavior_system, set_lookat_position_to_parent_transform_translation_behavior_system, CameraBodyTagComponent, CameraEyesTagComponent, CylinderCameraBodyBundle, CylinderCameraEyesBundle, CylinderCoordinatesForDesiredTransformTranslationVariablesComponent, DesiredTransformVariablesComponent, LookatVariablesComponent, ObservedEntityVariablesComponent, SetCylinderCoordinateForDesiredTransformTranslationUsingInputBehaviorComponent, SetDesiredTransformRotationToObservedEntityLocalUpBehaviorComponent, SetDesiredTransformTranslationToObservedEntityTransformTranslationBehaviorComponent
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
    CharacterPlayerInputComponent, CharacterTagComponent,
    CharacterTransformationFromPlayerToCameraVariablesComponent,
};
use math::CylindricalCoordinates;

use std::{f32::consts::PI, ops::Mul, time::Duration};

mod cylinder_camera;
mod character;
mod math;

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
    gltf_mesh_assets: Res<Assets<GltfMesh>>,
    mesh_assets: Res<Assets<Mesh>>,
) {
    if zone_loading_resource.did_spawn_main_gltf {
        return;
    }

    println!("spawn_scene_using_next_zone_resource_system");

    let main_zone_asset_handle = zone_loading_resource.main_gltf_asset_handle.clone();

    // if the GLTF has loaded, we can navigate its contents
    let gltf = gltf_assets.get(&main_zone_asset_handle).unwrap();
    let gltf_mesh = gltf_mesh_assets.get(&gltf.meshes[0]).unwrap();
    let mesh_handle = &gltf_mesh.primitives[0].mesh;
    let mesh = mesh_assets.get(mesh_handle.id()).unwrap();

    commands
        .spawn(TransformBundle {
            local: Transform::from_xyz(0.0, 0.0, 0.0),
            global: GlobalTransform::default(),
        })
        .with_children(|parent_commands| {
            // graphics
            parent_commands.spawn(SceneBundle {
                scene: gltf.scenes[0].clone(),
                ..Default::default()
            });

            // physics
            let collider =
                Collider::from_bevy_mesh(&mesh, &ComputedColliderShape::TriMesh).unwrap();
            parent_commands.spawn(collider);
        });

    zone_loading_resource.did_spawn_main_gltf = true;
}

fn update_character_rotation_from_player_to_character_system(
    mut character_query: Query<
        (
            &Transform,
            &mut CharacterTransformationFromPlayerToCameraVariablesComponent,
        ),
        With<CharacterTagComponent>,
    >,
    player_query: Query<(&Transform, &GlobalTransform), With<CameraEyesTagComponent>>,
) {
    let mut character = character_query.single_mut();
    let player = player_query.single();
    let character_up = character.0.up();
    let player_up = player.0.up();

    let rotation_from_player_up_to_character_up =
        Quat::from_rotation_arc(*player_up, *character_up);

    let next_transformation = Affine3A::mul(
        Affine3A::from_quat(rotation_from_player_up_to_character_up),
        player.1.affine(),
    );

    character
        .1
        .transformation_from_screen_to_global_on_character_horizontal = next_transformation;
}

fn apply_character_movement_input_using_player_input_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut character_query: Query<
        (
            &CharacterTransformationFromPlayerToCameraVariablesComponent,
            &mut CharacterPlayerInputComponent,
        ),
        With<CharacterTagComponent>,
    >,
) {
    let mut character = character_query.single_mut();

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

    let next_input = Affine3A::transform_vector3(
        &character
            .0
            .transformation_from_screen_to_global_on_character_horizontal,
        local_input,
    );

    character.1.global_movement_player_input = next_input;
}

fn apply_character_jump_input_using_player_input_system(
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
            &CharacterTransformationFromPlayerToCameraVariablesComponent,
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
}

fn draw_character_rotation_from_global_to_character_gizmos_system(
    mut gizmos: Gizmos,
    character_query: Query<
        (
            &Transform,
            &CharacterTransformationFromPlayerToCameraVariablesComponent,
        ),
        With<CharacterTagComponent>,
    >,
) {
    let character = character_query.single();

    let character_forward_input = Affine3A::transform_vector3(
        &character
            .1
            .transformation_from_screen_to_global_on_character_horizontal,
        Vec3::NEG_Z,
    );

    let character_right_input = Affine3A::transform_vector3(
        &character
            .1
            .transformation_from_screen_to_global_on_character_horizontal,
        Vec3::X,
    );

    gizmos.arrow(
        character.0.translation,
        character.0.translation + character_forward_input,
        Color::rgb(0.0, 1.0, 1.0),
    );
    gizmos.arrow(
        character.0.translation,
        character.0.translation + character_right_input,
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

fn setup_next_zone_to_suzanne_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut next_app_state: ResMut<NextState<AppState>>,
) {
    let asset_handle = asset_server.load::<Gltf>("zone/suzanne.glb");

    commands.insert_resource(NextZoneResource {
        asset_handles: Vec::from([asset_handle.clone().untyped()]),
        main_gltf_asset_handle: asset_handle,
        did_spawn_main_gltf: false,
    });

    next_app_state.set(AppState::LoadNextZone);
}

fn spawn_test_zone_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // zone, floor
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(25.0, 1.0, 25.0)),
            material: materials.add(Color::WHITE),
            transform: Transform::from_xyz(0.0, 99.5, 0.0),
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
            transform: Transform::from_xyz(0.0, 100.0, -5.0)
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
            transform: Transform::from_xyz(5.0, 100.5, 5.0),
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
            translation: Vec3::new(0.0, 0.0, 0.0),
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
                transform: Transform::from_xyz(0.0, 100.0, 0.0),
                inherited_visibility: InheritedVisibility::default(),
                rotation_from_player_to_character:
                    CharacterTransformationFromPlayerToCameraVariablesComponent {
                        transformation_from_screen_to_global_on_character_horizontal:
                            Affine3A::default(),
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
                Collider::capsule_y(0.5, 0.25),
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

fn spawn_camera_system(
    mut commands: Commands,
    query: Query<(Entity,), With<CharacterTagComponent>>,
) {
    // camera
    commands
        .spawn((CylinderCameraBodyBundle {
            tag: CameraBodyTagComponent,
            desired_transform_variables: DesiredTransformVariablesComponent {
                desired_transform: Transform::from_xyz(0.0, 0.0, 0.0),
            },
            observed_entity_variables: ObservedEntityVariablesComponent {
              entity: query.get_single().unwrap().0
            },
            set_desired_transform_translation_to_observed_entity_transform_translation: SetDesiredTransformTranslationToObservedEntityTransformTranslationBehaviorComponent,
            set_desired_transform_local_up_to_observed_entity_local_up: SetDesiredTransformRotationToObservedEntityLocalUpBehaviorComponent,
        },
      TransformBundle::from_transform(Transform::from_xyz(0.0, 0.0, 0.0))
      
      ))
        .with_children(|parent| {
            parent.spawn((
                CylinderCameraEyesBundle { tag: CameraEyesTagComponent, desired_transform_variables: DesiredTransformVariablesComponent {
                desired_transform: Transform::from_xyz(0.0, 0.0, 0.0),

                } , lookat_variables: LookatVariablesComponent {
                    position: Vec3::ZERO,
                    up: Vec3::Y,
                }, cylinder_coordindates_for_desired_transform_translation_variables: CylinderCoordinatesForDesiredTransformTranslationVariablesComponent {
                    cylinder_coordindates: CylindricalCoordinates {
                        distance: 25.0,
                        rotation: 0.0,
                        height: 5.0,
                    },
                }, set_cylinder_coordinate_for_desired_transform_translation_angle_using_input_behavior: SetCylinderCoordinateForDesiredTransformTranslationUsingInputBehaviorComponent,
                    set_lookat_position_to_parent_transform_translation_behavior: cylinder_camera::SetLookatPositionToParentTransformTranslationBehaviorComponent, },
                Camera3dBundle {
                    transform: Transform::from_xyz(0.0, 0., 0.0)
                        .looking_at(Vec3::new(0., 0., 0.), Vec3::Y),
                    ..default()
                },
            ));
        });
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

    app.add_systems(
        OnEnter(AppState::SetupNextZone),
        (
            spawn_scene_using_next_zone_resource_system,
            spawn_character_system,
            spawn_camera_system,
            spawn_test_zone_system,
        )
            .chain(),
    );

    app.add_plugins(DefaultPlugins);
    app.add_plugins((
        RapierPhysicsPlugin::<NoUserData>::default().in_fixed_schedule(),
        RapierDebugRenderPlugin::default(),
    ));

    app.add_systems(Startup, setup_next_zone_to_suzanne_system);

    app.add_systems(Update, transition_app_state_from_load_next_zone_to_setup_next_zone_when_next_zone_assets_loaded_system.run_if(in_state(AppState::LoadNextZone)));
    app.add_systems(
        Update,
        transition_app_state_from_setup_next_zone_to_play_when_zone_spawned_system
            .run_if(in_state(AppState::SetupNextZone)),
    );

    app.add_systems(
        FixedPreUpdate,
        (update_character_rotation_from_player_to_character_system)
            .run_if(in_state(AppState::Play))
            .before(CharacterPhaseMovementVelocitySystemSet),
    );

    app.add_systems(
        FixedPreUpdate,
        (
            update_character_body_try_jump_while_on_stage_system, // leave stage before calculating horizontal velocity
            update_character_horizontal_movement_velocity_system,
            update_character_movement_velocity_while_on_stage_system,
            update_character_movement_velocity_while_in_air_phase_system,
        )
            .in_set(CharacterPhaseMovementVelocitySystemSet)
            .chain()
            .run_if(in_state(AppState::Play)),
    );

    app.add_systems(
        FixedPreUpdate,
        (
            update_character_body_velocity_while_on_stage_using_movement_velocity_system,
            update_character_body_velocity_while_in_air_using_movement_velocity_system,
        )
            .chain()
            .after(CharacterPhaseMovementVelocitySystemSet)
            .run_if(in_state(AppState::Play)),
    );

    app.add_systems(
        FixedPreUpdate,
        (
            update_character_body_while_on_stage_system,
            update_character_body_try_land_while_in_air_system,
        )
            .before(PhysicsSet::StepSimulation)
            .run_if(in_state(AppState::Play)),
    );

    app.add_systems(
        Update,
        (
            apply_character_movement_input_using_player_input_system,
            apply_character_jump_input_using_player_input_system,
        )
            .run_if(in_state(AppState::Play)),
    );

        app.add_systems(
        FixedUpdate,
        (
            transition_desired_transform_to_transform_system,
            apply_desired_transform_using_cylinder_coordinates_system,
        )
            .run_if(in_state(AppState::Play)),
    );


    app.add_systems(
        Update,
        (
            apply_lookat_to_transform_system,
            set_desired_transform_translation_to_observed_entiy_transform_translation_behavior_system,
            set_desired_transform_rotation_to_observed_entity_local_up_behavior_system,
            set_lookat_position_to_parent_transform_translation_behavior_system,
            set_cylinder_coordinates_for_desired_transform_translation_using_input_system
        )
            .run_if(in_state(AppState::Play)),
    );

    app.add_systems(
        PostUpdate,
        (
            draw_character_rotation_from_global_to_character_gizmos_system,
            draw_character_transform_gizmos_system,
            draw_character_input_gizmos_system,
            draw_character_body_velocity_gizmos_system,
            draw_character_horizontal_movement_velocity_gizmos_system,
            draw_character_vertical_movement_velocity_gizmos_system,
        )
            .run_if(in_state(AppState::Play)),
    );

    app.run();
}
