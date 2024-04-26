use std::f32::consts::{FRAC_PI_4, PI};

use bevy::{
    app::{App, FixedUpdate, Startup, Update},
    asset::Assets,
    core_pipeline::core_3d::Camera3dBundle,
    ecs::{
        bundle::Bundle,
        component::Component,
        event::EventReader,
        query::With,
        system::{Commands, Query, Res, ResMut},
    },
    gizmos::gizmos::Gizmos,
    input::{
        mouse::{MouseButton, MouseMotion},
        ButtonInput,
    },
    math::{
        primitives::{Capsule3d, Circle, Cuboid},
        Quat, Vec3,
    },
    pbr::{
        light_consts, AlphaMode, AmbientLight, CascadeShadowConfigBuilder, DirectionalLight,
        DirectionalLightBundle, PbrBundle, StandardMaterial,
    },
    render::{color::Color, mesh::Mesh},
    transform::components::Transform,
    utils::default,
    DefaultPlugins,
};

// region character

#[derive(Component)]
struct CharacterIsComponent;

#[derive(Component)]
struct RotationFromPlayerCameraToCharacterComponent {
    rotation_quat: Quat,
}

#[derive(Bundle)]
struct CharacterBundle {
    is: CharacterIsComponent,
    rotation_from_player_camera_to_character: RotationFromPlayerCameraToCharacterComponent,
}

// endregion

// region camera

#[derive(Component)]
struct PlayerCameraIsComponent;

#[derive(Component)]
struct PlayerCameraSphericalCoordinates {
    radius: f32,
    /**
     * Polar angle in radians from the y (up) axis.
     * "phi".
     */
    phi: f32,
    /**
     * equator angle in radians around the y (up) axis.
     */
    theta: f32,
}

#[derive(Component)]
struct PlayerCameraRollComponent(pub f32);

#[derive(Bundle)]
struct PlayerCameraBundle {
    is: PlayerCameraIsComponent,
    spherical_coordinates: PlayerCameraSphericalCoordinates,
    roll: PlayerCameraRollComponent,
}

fn update_player_camera_coordinates_using_input_system(
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut player_camera_query: Query<
        &mut PlayerCameraSphericalCoordinates,
        With<PlayerCameraIsComponent>,
    >,
) {
    let mut player_camera = player_camera_query.single_mut();
    for event in mouse_motion_events.read() {
        player_camera.theta += event.delta.x * 0.001;
        player_camera.phi = player_camera.phi - event.delta.y * 0.001
    }
}

fn update_player_camera_roll_using_input_system(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut player_camera_query: Query<&mut PlayerCameraRollComponent, With<PlayerCameraIsComponent>>,
) {
    let mut player_camera = player_camera_query.single_mut();

    if mouse_button_input.pressed(MouseButton::Left) {
        player_camera.0 -= 0.01
    }

    if mouse_button_input.pressed(MouseButton::Right) {
        player_camera.0 += 0.01
    }
}

fn update_player_camera_to_character_rotation_using_coordinates_system(
    mut player_camera_query: Query<
        (
            &mut Transform,
            &PlayerCameraSphericalCoordinates,
            &PlayerCameraRollComponent,
        ),
        With<PlayerCameraIsComponent>,
    >,
) {
    let mut player_camera = player_camera_query.single_mut();

    let sin_phi_radius = f32::sin(player_camera.1.phi) * player_camera.1.radius;

    player_camera.0.translation = Vec3::new(
        sin_phi_radius * f32::sin(player_camera.1.theta),
        f32::cos(player_camera.1.phi) * player_camera.1.radius,
        sin_phi_radius * f32::cos(player_camera.1.theta),
    );

    player_camera.0.look_at(Vec3::ZERO, Vec3::Y);
    player_camera.0.rotate_local_z(player_camera.2 .0);
}

fn update_character_rotation_transformation_system(
    mut character_query: Query<
        (
            &Transform,
            &mut RotationFromPlayerCameraToCharacterComponent,
        ),
        With<CharacterIsComponent>,
    >,
    player_camera_query: Query<&Transform, With<PlayerCameraIsComponent>>,
) {
    let mut character = character_query.single_mut();
    let player_camera = player_camera_query.single();
    let camera_up = player_camera.up();
    let character_up = character.0.up();

    character.1.rotation_quat = Quat::from_rotation_arc(*camera_up, *character_up);
}

// endregion

// region debug systems

fn draw_gizmos_system(
    mut gizmos: Gizmos,
    player_camera_query: Query<&Transform, With<PlayerCameraIsComponent>>,
    character_query: Query<
        (&Transform, &RotationFromPlayerCameraToCharacterComponent),
        With<CharacterIsComponent>,
    >,
) {
    let player_camera = player_camera_query.single();
    let character = character_query.single();
    gizmos.arrow(
        character.0.translation,
        Quat::mul_vec3(character.1.rotation_quat, *player_camera.forward()),
        Color::BLUE,
    );
    gizmos.arrow(
        character.0.translation,
        Quat::mul_vec3(character.1.rotation_quat, *player_camera.right()),
        Color::RED,
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
        material: materials.add(Color::WHITE),
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
    commands.spawn((
        CharacterBundle {
            is: CharacterIsComponent,
            rotation_from_player_camera_to_character:
                RotationFromPlayerCameraToCharacterComponent {
                    rotation_quat: Quat::IDENTITY,
                },
        },
        PbrBundle {
            mesh: meshes.add(Capsule3d::new(0.5, 1.)),
            material: materials.add(StandardMaterial {
                base_color: Color::rgba(0.0, 0.0, 1.0, 0.5),
                alpha_mode: AlphaMode::Blend,
                ..default()
            }),
            ..default()
        },
    ));
}

fn spawn_player_camera_system(mut commands: Commands) {
    // camera
    commands.spawn((
        PlayerCameraBundle {
            is: PlayerCameraIsComponent,
            spherical_coordinates: PlayerCameraSphericalCoordinates {
                radius: 25.0,
                phi: FRAC_PI_4,
                theta: FRAC_PI_4,
            },
            roll: PlayerCameraRollComponent(0.0),
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
        .add_systems(Startup, spawn_player_camera_system)
        .add_systems(Startup, spawn_props_system)
        .add_systems(
            FixedUpdate,
            update_player_camera_coordinates_using_input_system,
        )
        .add_systems(FixedUpdate, update_player_camera_roll_using_input_system)
        .add_systems(FixedUpdate, update_character_rotation_transformation_system)
        .add_systems(
            FixedUpdate,
            update_player_camera_to_character_rotation_using_coordinates_system,
        )
        .add_systems(Update, draw_gizmos_system)
        .run();
}
