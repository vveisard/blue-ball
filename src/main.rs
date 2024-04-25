use std::f32::consts::FRAC_PI_4;

use bevy::{
    app::{App, FixedUpdate, Startup, Update},
    asset::Assets,
    core_pipeline::core_3d::Camera3dBundle,
    ecs::{
        bundle::Bundle,
        component::Component,
        event::EventReader,
        query::With,
        system::{Commands, Query, ResMut},
    },
    gizmos::gizmos::Gizmos,
    input::mouse::MouseMotion,
    math::{primitives::Capsule3d, Mat4, Quat, Vec3},
    pbr::{PbrBundle, StandardMaterial},
    render::{color::Color, mesh::Mesh},
    transform::components::Transform,
    utils::default,
    DefaultPlugins,
};

// region character

#[derive(Component)]
struct CharacterIsComponent;

#[derive(Component)]
struct TransformFromPlayerCameraToCharacterComponent {
    matrix: Mat4,
}

#[derive(Bundle)]
struct CharacterBundle {
    is: CharacterIsComponent,
    transform_from_player_camera_to_character: TransformFromPlayerCameraToCharacterComponent,
}

// endregion

fn update_camera_transformation_system(
    mut character_query: Query<
        (
            &Transform,
            &mut TransformFromPlayerCameraToCharacterComponent,
        ),
        With<CharacterIsComponent>,
    >,
    player_camera_query: Query<&Transform, With<PlayerCameraIsComponent>>,
) {
    let mut character = character_query.single_mut();
    let player_camera = player_camera_query.single();

    let rotation: Quat =
        Quat::from_rotation_arc(Vec3::from(player_camera.up()), Vec3::from(character.0.up()));

    character.1.matrix = Mat4::from_quat(rotation);
}

fn draw_gizmos_system(mut gizmos: Gizmos) {
    gizmos.arrow(Vec3::ZERO, Vec3::Z * 5., Color::YELLOW);
}

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

#[derive(Bundle)]
struct PlayerCameraBundle {
    is: PlayerCameraIsComponent,
    spherical_coordinates: PlayerCameraSphericalCoordinates,
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
        // player_camera.phi = f32::clamp(
        //     player_camera.phi + event.delta.y * 0.001,
        //     FRAC_PI_6,
        //     PI - FRAC_PI_6,
        // );
    }
}

fn update_player_camera_translation_using_coordinates_system(
    mut player_camera_query: Query<
        (&mut Transform, &PlayerCameraSphericalCoordinates),
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

    player_camera.0.look_at(Vec3::ZERO, Vec3::Y)
}

// endregion

// region startup

fn spawn_character_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        CharacterBundle {
            is: CharacterIsComponent,
            transform_from_player_camera_to_character:
                TransformFromPlayerCameraToCharacterComponent {
                    matrix: Transform::IDENTITY.compute_matrix(),
                },
        },
        PbrBundle {
            mesh: meshes.add(Capsule3d::new(1.0, 2.)),
            material: materials.add(Color::BLUE),
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
        .add_systems(FixedUpdate, update_camera_transformation_system)
        .add_systems(Update, update_player_camera_coordinates_using_input_system)
        .add_systems(
            Update,
            update_player_camera_translation_using_coordinates_system,
        )
        .add_systems(Update, draw_gizmos_system)
        .run();
}
