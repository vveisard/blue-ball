use bevy::{
    app::{App, FixedUpdate, Startup, Update},
    asset::Assets,
    core_pipeline::core_3d::Camera3dBundle,
    ecs::{
        bundle::Bundle,
        component::Component,
        event::EventReader,
        query::{With, Without},
        system::{Commands, Query, Res, ResMut},
    },
    gizmos::gizmos::Gizmos,
    hierarchy::BuildChildren,
    input::{
        keyboard::KeyCode,
        mouse::{MouseButton, MouseMotion},
        ButtonInput,
    },
    math::{
        primitives::{Capsule3d, Circle, Cuboid},
        Quat, Vec2, Vec3,
    },
    pbr::{
        light_consts, AlphaMode, AmbientLight, CascadeShadowConfigBuilder, DirectionalLight,
        DirectionalLightBundle, PbrBundle, StandardMaterial,
    },
    render::{color::Color, mesh::Mesh, view::InheritedVisibility},
    transform::components::{GlobalTransform, Transform},
    utils::default,
    DefaultPlugins,
};
use std::f32::consts::PI;

// region character

#[derive(Component)]
struct CharacterTagComponent;

/// Character component.
#[derive(Component)]
struct CharacterRotationFromGlobalToCharacterParametersComponent {
    /// rotation from global space to character space
    rotation_from_global_to_character_quat: Quat,
}

/// component with input from player for character.
#[derive(Component)]
struct CharacterPlayerInputComponent {
    /// player input vector in global space
    /// ie, input in camera space transformed to global space
    global_input: Vec3,
}

#[derive(Bundle)]
struct CharacterBundle {
    tag: CharacterTagComponent,
    global_transform: GlobalTransform,
    transform: Transform,
    inherited_visibility: InheritedVisibility,
    rotation_from_player_to_character: CharacterRotationFromGlobalToCharacterParametersComponent,
    player_input: CharacterPlayerInputComponent,
}

// endregion

// region camera

#[derive(Component)]
struct PlayerTagComponent;

/// cordinates to
struct CylinderCoordinates3d {
    // dTagtance to the center
    distance: f32,
    // rotation about the center
    rotation: f32,
    // height about the cylinder
    height: f32,
}

#[derive(Component)]
struct PlayerCameraCharacterParametersComponent {
    /// translation with respect to the character
    translation_cylinder_coordinates: CylinderCoordinates3d,
    // point to lookat
    focus: Vec3,
}

#[derive(Component)]
struct PlayerCameraRollParametersComponent(pub f32);

#[derive(Bundle)]
struct PlayerBundle {
    tag: PlayerTagComponent,
    camera_character_parameters: PlayerCameraCharacterParametersComponent,
    camera_roll_parameters: PlayerCameraRollParametersComponent,
}

fn update_player_roll_using_input_system(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut player_query: Query<&mut PlayerCameraRollParametersComponent, With<PlayerTagComponent>>,
) {
    let mut player = player_query.single_mut();

    if mouse_button_input.pressed(MouseButton::Left) {
        player.0 -= 0.01
    }

    if mouse_button_input.pressed(MouseButton::Right) {
        player.0 += 0.01
    }
}

fn update_player_local_character_camera_parameters_using_input_system(
    mut mouse_events: EventReader<MouseMotion>,
    mut player_query: Query<
        (
            &mut Transform,
            &mut PlayerCameraCharacterParametersComponent,
        ),
        (With<PlayerTagComponent>, Without<CharacterTagComponent>),
    >,
) {
    let mut player = player_query.single_mut();

    let mut input = Vec2::ZERO;
    for mouse_event in mouse_events.read() {
        input.x += mouse_event.delta.x * 0.001;
        input.y += mouse_event.delta.y * 0.001;
    }

    player.1.translation_cylinder_coordinates.rotation += input.x;
    player.1.focus.y -= input.y;
    player.1.translation_cylinder_coordinates.height -= input.y * 0.5;
}

fn update_player_translation_system(
    mut character_query: Query<
        (&Transform,),
        (With<CharacterTagComponent>, Without<PlayerTagComponent>),
    >,
    mut player_query: Query<
        (
            &mut Transform,
            &PlayerCameraCharacterParametersComponent,
            &PlayerCameraRollParametersComponent,
        ),
        (With<PlayerTagComponent>, Without<CharacterTagComponent>),
    >,
) {
    let character = character_query.single_mut();
    let mut player = player_query.single_mut();

    let relative_x_translation = player.1.translation_cylinder_coordinates.distance
        * f32::cos(player.1.translation_cylinder_coordinates.rotation);
    let relative_z_translation = player.1.translation_cylinder_coordinates.distance
        * f32::sin(player.1.translation_cylinder_coordinates.rotation);
    let relative_y_translation = player.1.translation_cylinder_coordinates.height;

    player.0.translation = character.0.translation
        + Vec3::new(
            relative_x_translation,
            relative_y_translation,
            relative_z_translation,
        );

    player
        .0
        .look_at(character.0.translation + player.1.focus, Vec3::Y);

    player.0.rotate_local_z(player.2 .0);
}

// ThTag system prints messages when you press or release the left mouse button:
fn mouse_player_roll_on_mouse_button_input(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut player_query: Query<&mut PlayerCameraRollParametersComponent, With<PlayerTagComponent>>,
) {
    if !mouse_button_input.just_pressed(MouseButton::Middle) {
        return;
    }

    player_query.single_mut().0 = 0.0;
}

// endregion

// region

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

    // taken from https://github.com/bevyengine/bevy/dTagcussions/8501
    character.0.global_input = player_global_transform
        .affine()
        .transform_vector3(local_input)
}

// endregion

// region debug systems

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
        Color::BLUE,
    );
    gizmos.arrow(
        character.0.translation,
        character.0.translation
            + Quat::mul_vec3(
                character.1.rotation_from_global_to_character_quat,
                *player.right(),
            ),
        Color::RED,
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
    // circular base
    commands.spawn(PbrBundle {
        mesh: meshes.add(Circle::new(4.0)),
        material: materials.add(StandardMaterial {
            base_color: Color::WHITE,
            cull_mode: None,
            ..default()
        }),
        transform: Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        ..default()
    });
    // cube
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
        material: materials.add(Color::WHITE),
        transform: Transform::from_xyz(5.0, 0.5, 5.0),
        ..default()
    });

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
        .spawn((CharacterBundle {
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
        },))
        .with_children(|parent| {
            parent.spawn(PbrBundle {
                transform: Transform::from_xyz(0.0, 1.0, 0.0),
                mesh: meshes.add(Capsule3d::new(0.5, 1.)),
                material: materials.add(StandardMaterial {
                    base_color: Color::rgba(0.0, 0.0, 1.0, 0.5),
                    alpha_mode: AlphaMode::Blend,
                    ..default()
                }),
                ..default()
            });
        });
}

fn spawn_player_system(mut commands: Commands) {
    // camera
    commands.spawn((
        PlayerBundle {
            tag: PlayerTagComponent,
            camera_roll_parameters: PlayerCameraRollParametersComponent(0.0),
            camera_character_parameters: PlayerCameraCharacterParametersComponent {
                translation_cylinder_coordinates: CylinderCoordinates3d {
                    distance: 15.0,
                    rotation: 0.0,
                    height: 5.0,
                },
                focus: Vec3::new(0.0, 4.0, 0.0),
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
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, spawn_character_system)
        .add_systems(Startup, spawn_player_system)
        .add_systems(Startup, spawn_props_system)
        .add_systems(FixedUpdate, update_player_roll_using_input_system)
        .add_systems(
            FixedUpdate,
            update_player_local_character_camera_parameters_using_input_system,
        )
        .add_systems(
            FixedUpdate,
            update_character_rotation_from_player_to_character_system,
        )
        .add_systems(FixedUpdate, update_character_movement_player_input_system)
        .add_systems(FixedUpdate, update_player_translation_system)
        .add_systems(Update, mouse_player_roll_on_mouse_button_input)
        .add_systems(
            Update,
            draw_character_rotation_from_global_to_character_gizmos_system,
        )
        .add_systems(Update, draw_character_input_gizmos_system)
        .run();
}
