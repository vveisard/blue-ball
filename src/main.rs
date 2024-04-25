use bevy::{
    app::{App, Startup},
    asset::Assets,
    core_pipeline::core_3d::Camera3dBundle,
    ecs::{
        bundle::Bundle,
        component::Component,
        system::{Commands, ResMut},
    },
    math::{primitives::Capsule3d, Vec3},
    pbr::{PbrBundle, StandardMaterial},
    render::{color::Color, mesh::Mesh},
    transform::components::Transform,
    utils::default,
    DefaultPlugins,
};

// region character

#[derive(Component)]
struct IsCharacterComponent;

#[derive(Bundle)]
struct CharacterBundle {
    is: IsCharacterComponent,
}

// endregion

// region camera

#[derive(Component)]
struct IsPlayerCameraComponent;

#[derive(Bundle)]
struct PlayerCameraBundle {
    is: IsCharacterComponent,
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
        },
        PbrBundle {
            mesh: meshes.add(Capsule3d::new(1.0, 2.)),
            material: materials.add(Color::BLUE),
            ..default()
        },
    ));
}

fn spawn_camera_system(mut commands: Commands) {
    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 25., 25.).looking_at(Vec3::new(0., 0., 0.), Vec3::Y),
        ..default()
    });
}

// endregion

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, spawn_character_system)
        .add_systems(Startup, spawn_camera_system)
        .run();
}
