use bevy::{
    app::{App, FixedUpdate, Startup, Update},
    asset::Assets,
    core_pipeline::core_3d::Camera3dBundle,
    ecs::{
        bundle::Bundle,
        component::Component,
        query::With,
        system::{Commands, Query, ResMut},
    },
    gizmos::gizmos::Gizmos,
    math::{primitives::Capsule3d, Mat4, Quat, Vec3},
    pbr::{PbrBundle, StandardMaterial},
    render::{color::Color, mesh::Mesh},
    transform::components::Transform,
    utils::default,
    DefaultPlugins,
};

// region character

#[derive(Component)]
struct IsCharacterComponent;

#[derive(Component)]
struct TransformFromPlayerCameraToCharacterComponent {
    matrix: Mat4,
}

#[derive(Bundle)]
struct CharacterBundle {
    is: IsCharacterComponent,
    transform_from_player_camera_to_character: TransformFromPlayerCameraToCharacterComponent,
}

// endregion

fn update_camera_transformation(
    mut character_query: Query<
        (
            &Transform,
            &mut TransformFromPlayerCameraToCharacterComponent,
        ),
        With<IsCharacterComponent>,
    >,
    player_camera_query: Query<&Transform, With<IsPlayerCameraComponent>>,
) {
    let mut character = character_query.single_mut();
    let player_camera = player_camera_query.single();

    let rotation: Quat =
        Quat::from_rotation_arc(Vec3::from(player_camera.up()), Vec3::from(character.0.up()));

    character.1.matrix = Mat4::from_quat(rotation);
}

fn draw_gizmos_system(mut gizmos: Gizmos) {
    gizmos.arrow(Vec3::ZERO, Vec3::Z, Color::YELLOW);
}

// region camera

#[derive(Component)]
struct IsPlayerCameraComponent;

#[derive(Bundle)]
struct PlayerCameraBundle {
    is: IsPlayerCameraComponent,
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
            is: IsCharacterComponent,
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
            is: IsPlayerCameraComponent,
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
        .add_systems(FixedUpdate, update_camera_transformation)
        .add_systems(Update, draw_gizmos_system)
        .run();
}
