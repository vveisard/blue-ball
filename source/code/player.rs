use crate::{
    character::{
        CharacterPlayerInputComponent, CharacterTransformationFromPlayerToCameraVariablesComponent,
    },
    math::{
        CylinderCoordinates3dSmoothDampTransitionVariables, CylindricalCoordinates,
        FromCylindrical, SmoothDampTransitionVariables,
    },
};
use bevy::{
    ecs::{
        bundle::Bundle,
        component::Component,
        event::EventReader,
        query::{With, Without},
        system::{Query, Res},
    },
    gizmos::gizmos::Gizmos,
    input::{
        mouse::{MouseButton, MouseMotion, MouseWheel},
        ButtonInput,
    },
    math::{primitives::Direction3d, Quat, Vec2, Vec3},
    render::color::Color,
    time::Time,
    transform::components::Transform,
};

use crate::character::CharacterTagComponent;
use crate::math::SmoothDamp;

#[derive(Component)]
pub struct PlayerTagComponent;

/// cordinates to

/// variables of transition for state of player camera.
#[derive(Component)]
pub struct PlayerCameraTransitionVariablesComponent {
    pub origin_translation: SmoothDampTransitionVariables<Vec3>,
    pub eyes_translation: CylinderCoordinates3dSmoothDampTransitionVariables,
    pub eyes_lookat: SmoothDampTransitionVariables<Vec3>,
    pub eyes_roll: SmoothDampTransitionVariables<f32>,
}

// TODO transition state parameters

pub struct PlayerCameraState {
    /// translation of origin, in world space.
    pub origin_translation: Vec3,
    /// translation of eyes, with respect to origin.
    pub eyes_translation: CylindricalCoordinates,
    /// point to lookat, relative to origin_translation
    pub eyes_lookat: Vec3,

    pub eyes_roll: f32,
}

/// current state of player camera.
#[derive(Component)]
pub struct PlayerCameraTransitionCurrentStateComponent {
    pub camera_state: PlayerCameraState,
}

/// desired state of player camera.
#[derive(Component)]
pub struct PlayerCameraTransitionDesiredStateComponent {
    pub camera_state: PlayerCameraState,
}

#[derive(Bundle)]
pub struct PlayerBundle {
    pub tag: PlayerTagComponent,
    pub camera_transition_current_state: PlayerCameraTransitionCurrentStateComponent,
    pub camera_transition_desired_state: PlayerCameraTransitionDesiredStateComponent,
    pub camera_transition_variables: PlayerCameraTransitionVariablesComponent,
}

// REGION update transition desired state

/// update desired translation for player camera origin.
pub fn set_player_camera_origin_desired_state_translation_using_character_system(
    mut player_query: Query<
        (&mut PlayerCameraTransitionDesiredStateComponent,),
        (With<PlayerTagComponent>,),
    >,
    character_query: Query<(&Transform,), (With<CharacterTagComponent>,)>,
) {
    let character = character_query.single();
    let mut player = player_query.single_mut();

    player.0.camera_state.origin_translation = character.0.translation;
}

/// update desired roll for player camera eyes.
pub fn set_player_camera_eyes_desired_state_roll_using_input_system(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut player_query: Query<
        (&mut PlayerCameraTransitionDesiredStateComponent,),
        With<PlayerTagComponent>,
    >,
) {
    let mut player = player_query.single_mut();

    if mouse_button_input.pressed(MouseButton::Left) {
        player.0.camera_state.eyes_roll -= 0.01
    }

    if mouse_button_input.pressed(MouseButton::Right) {
        player.0.camera_state.eyes_roll += 0.01
    }
}

/// update desired height and lookat for player camera eyes.
pub fn set_player_camera_eyes_desired_state_height_and_lookat_using_input_system(
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut player_query: Query<
        (
            &mut Transform,
            &mut PlayerCameraTransitionDesiredStateComponent,
        ),
        (With<PlayerTagComponent>, Without<CharacterTagComponent>),
    >,
) {
    let mut player = player_query.single_mut();

    let mut input = Vec2::ZERO;
    for mouse_event in mouse_motion_events.read() {
        input.x += mouse_event.delta.x * 0.001;
        input.y += mouse_event.delta.y * 0.001;
    }

    let mut zoom_input: f32 = 0.0;
    for mouse_event in mouse_wheel_events.read() {
        zoom_input += mouse_event.y * 0.1;
    }

    player.1.camera_state.eyes_translation.distance -= zoom_input;

    player.1.camera_state.eyes_translation.rotation += input.x;
    player.1.camera_state.eyes_lookat.y -= input.y;
    player.1.camera_state.eyes_translation.height -= input.y * 0.5;
}

pub fn set_player_camera_eyes_desired_state_roll_on_mouse_input_system(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut player_query: Query<
        (&mut PlayerCameraTransitionDesiredStateComponent,),
        With<PlayerTagComponent>,
    >,
) {
    if !mouse_button_input.just_pressed(MouseButton::Middle) {
        return;
    }

    player_query.single_mut().0.camera_state.eyes_roll = 0.0;
}

// REGIONEND

// REGION transition current state systems

pub fn transition_player_camera_origin_current_state_system(
    time: Res<Time>,
    mut player_query: Query<
        (
            &PlayerCameraTransitionDesiredStateComponent,
            &mut PlayerCameraTransitionVariablesComponent,
            &mut PlayerCameraTransitionCurrentStateComponent,
        ),
        (With<PlayerTagComponent>,),
    >,
) {
    for (
        player_camera_desired_state,
        mut player_camera_transition_state_variables,
        mut player_camera_current_state,
    ) in &mut player_query
    {
        let smooth_damp_result = Vec3::smooth_damp(
            player_camera_current_state.camera_state.origin_translation,
            player_camera_desired_state.camera_state.origin_translation,
            player_camera_transition_state_variables
                .origin_translation
                .velocity,
            0.1,
            f32::INFINITY,
            time.delta().as_secs_f32(),
        );

        player_camera_current_state.camera_state.origin_translation = smooth_damp_result.0;

        player_camera_transition_state_variables
            .origin_translation
            .velocity = smooth_damp_result.1;
    }
}

/// transition player camera eyes current state towards desired state.
pub fn transition_player_camera_eyes_current_state_rotation_system(
    time: Res<Time>,
    mut player_query: Query<
        (
            &PlayerCameraTransitionDesiredStateComponent,
            &mut PlayerCameraTransitionVariablesComponent,
            &mut PlayerCameraTransitionCurrentStateComponent,
        ),
        (With<PlayerTagComponent>,),
    >,
) {
    for (
        player_camera_desired_state,
        mut player_camera_transition_state_variables,
        mut player_camera_current_state,
    ) in &mut player_query
    {
        let smooth_damp_result = f32::smooth_damp(
            player_camera_current_state
                .camera_state
                .eyes_translation
                .rotation,
            player_camera_desired_state
                .camera_state
                .eyes_translation
                .rotation,
            player_camera_transition_state_variables
                .eyes_translation
                .rotation
                .velocity,
            0.1,
            f32::INFINITY,
            time.delta().as_secs_f32(),
        );

        player_camera_current_state
            .camera_state
            .eyes_translation
            .rotation = smooth_damp_result.0;

        player_camera_transition_state_variables
            .eyes_translation
            .rotation
            .velocity = smooth_damp_result.1;
    }
}

/// transition player camera eyes current state towards desired state.
pub fn transition_player_camera_eyes_current_state_height_system(
    time: Res<Time>,
    mut player_query: Query<
        (
            &PlayerCameraTransitionDesiredStateComponent,
            &mut PlayerCameraTransitionVariablesComponent,
            &mut PlayerCameraTransitionCurrentStateComponent,
        ),
        (With<PlayerTagComponent>,),
    >,
) {
    for (
        player_camera_desired_state,
        mut player_camera_transition_state_variables,
        mut player_camera_current_state,
    ) in &mut player_query
    {
        let smooth_damp_result = f32::smooth_damp(
            player_camera_current_state
                .camera_state
                .eyes_translation
                .height,
            player_camera_desired_state
                .camera_state
                .eyes_translation
                .height,
            player_camera_transition_state_variables
                .eyes_translation
                .height
                .velocity,
            0.1,
            f32::INFINITY,
            time.delta().as_secs_f32(),
        );

        player_camera_current_state
            .camera_state
            .eyes_translation
            .height = smooth_damp_result.0;

        player_camera_transition_state_variables
            .eyes_translation
            .height
            .velocity = smooth_damp_result.1;
    }
}

/// transition player camera eyes current state towards desired state.
pub fn transition_player_camera_eyes_current_state_distance_system(
    time: Res<Time>,
    mut player_query: Query<
        (
            &PlayerCameraTransitionDesiredStateComponent,
            &mut PlayerCameraTransitionVariablesComponent,
            &mut PlayerCameraTransitionCurrentStateComponent,
        ),
        (With<PlayerTagComponent>,),
    >,
) {
    for (
        player_camera_desired_state,
        mut player_camera_transition_state_variables,
        mut player_camera_current_state,
    ) in &mut player_query
    {
        let smooth_damp_result = f32::smooth_damp(
            player_camera_current_state
                .camera_state
                .eyes_translation
                .distance,
            player_camera_desired_state
                .camera_state
                .eyes_translation
                .distance,
            player_camera_transition_state_variables
                .eyes_translation
                .distance
                .velocity,
            0.1,
            f32::INFINITY,
            time.delta().as_secs_f32(),
        );

        player_camera_current_state
            .camera_state
            .eyes_translation
            .distance = smooth_damp_result.0;

        player_camera_transition_state_variables
            .eyes_translation
            .distance
            .velocity = smooth_damp_result.1;
    }
}

/// transition player camera eyes current state towards desired state.
pub fn transition_player_camera_eyes_current_state_roll_system(
    time: Res<Time>,
    mut player_query: Query<
        (
            &PlayerCameraTransitionDesiredStateComponent,
            &mut PlayerCameraTransitionVariablesComponent,
            &mut PlayerCameraTransitionCurrentStateComponent,
        ),
        (With<PlayerTagComponent>,),
    >,
) {
    for (
        player_camera_desired_state,
        mut player_camera_transition_state_variables,
        mut player_camera_current_state,
    ) in &mut player_query
    {
        let smooth_damp_result = f32::smooth_damp(
            player_camera_current_state.camera_state.eyes_roll,
            player_camera_desired_state.camera_state.eyes_roll,
            player_camera_transition_state_variables.eyes_roll.velocity,
            0.1,
            f32::INFINITY,
            time.delta().as_secs_f32(),
        );

        player_camera_current_state.camera_state.eyes_roll = smooth_damp_result.0;

        player_camera_transition_state_variables.eyes_roll.velocity = smooth_damp_result.1;
    }
}

/// transition player camera eyes current state towards desired state.
pub fn transition_player_camera_eyes_current_state_lookat_system(
    time: Res<Time>,
    mut player_query: Query<
        (
            &PlayerCameraTransitionDesiredStateComponent,
            &mut PlayerCameraTransitionVariablesComponent,
            &mut PlayerCameraTransitionCurrentStateComponent,
        ),
        (With<PlayerTagComponent>,),
    >,
) {
    for (
        player_camera_desired_state,
        mut player_camera_transition_state_variables,
        mut player_camera_current_state,
    ) in &mut player_query
    {
        let smooth_damp_result = Vec3::smooth_damp(
            player_camera_current_state.camera_state.eyes_lookat,
            player_camera_desired_state.camera_state.eyes_lookat,
            player_camera_transition_state_variables
                .eyes_lookat
                .velocity,
            0.1,
            f32::INFINITY,
            time.delta().as_secs_f32(),
        );

        player_camera_current_state.camera_state.eyes_lookat = smooth_damp_result.0;

        player_camera_transition_state_variables
            .eyes_lookat
            .velocity = smooth_damp_result.1;
    }
}

// REGIONEND

// REGION update state (writeback to core components)

/// apply player camera eyes transform using current state system.
pub fn apply_player_camera_transform_using_current_state_system(
    mut player_query: Query<
        (&PlayerCameraTransitionCurrentStateComponent, &mut Transform),
        (With<PlayerTagComponent>, Without<CharacterTagComponent>),
    >,
) {
    let mut player = player_query.single_mut();
    let next_origin_translation: Vec3 = player.0.camera_state.origin_translation.clone();
    let next_eyes_translation = Vec3::from_cylindrical(&player.0.camera_state.eyes_translation);

    player.1.translation = next_origin_translation + next_eyes_translation;

    player.1.look_at(
        next_origin_translation + player.0.camera_state.eyes_lookat,
        Vec3::Y,
    );

    player.1.rotate_local_z(player.0.camera_state.eyes_roll);
}

// REGIONEND

pub fn draw_player_camera_lookat_gizmos_system(
    mut gizmos: Gizmos,
    player_query: Query<
        (
            &PlayerCameraTransitionCurrentStateComponent,
            &PlayerCameraTransitionDesiredStateComponent,
        ),
        With<PlayerTagComponent>,
    >,
) {
    let player = player_query.single();

    gizmos.sphere(
        player.0.camera_state.origin_translation + player.0.camera_state.eyes_lookat,
        Quat::IDENTITY,
        0.5,
        Color::WHITE,
    );

    gizmos.sphere(
        player.1.camera_state.origin_translation + player.1.camera_state.eyes_lookat,
        Quat::IDENTITY,
        0.5,
        Color::YELLOW,
    );
}

pub fn draw_player_camera_gizmos_system(
    mut gizmos: Gizmos,
    player_query: Query<(&PlayerCameraTransitionCurrentStateComponent,), With<PlayerTagComponent>>,
) {
    let player = player_query.single();

    let player_camera_origin_current_translation = player.0.camera_state.origin_translation;
    let player_camera_eyes_current_translation =
        Vec3::from_cylindrical(&player.0.camera_state.eyes_translation);

    gizmos.circle(
        player_camera_origin_current_translation,
        Direction3d::Y,
        player.0.camera_state.eyes_translation.distance,
        Color::WHITE,
    );

    gizmos.line(
        player_camera_origin_current_translation,
        player_camera_origin_current_translation
            + Vec3::new(0.0, player_camera_eyes_current_translation.y, 0.0),
        Color::WHITE,
    );

    gizmos.line(
        player_camera_origin_current_translation,
        player_camera_origin_current_translation
            + Vec3::new(
                player_camera_eyes_current_translation.x,
                0.0,
                player_camera_eyes_current_translation.z,
            ),
        Color::WHITE,
    );
}
