use std::f32::consts::PI;

use crate::math::{
    CylinderCoordinates3dSmoothDampTransitionVariables, CylindricalCoordinates, FromCylindrical,
    LerpAngle, Slerp, SmoothDampTransitionVariables,
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

const CAMERA_TRANSITON_SPEED: f32 = 2.5;

#[derive(Component)]
pub struct PlayerTagComponent;

/// cordinates to

/// transition variables for "cylinder" behavior of player camera.
#[derive(Component)]
pub struct PlayerCameraCylinderTransitionVariablesComponent {
    pub origin_translation: SmoothDampTransitionVariables<Vec3>,
    pub origin_rotation: SmoothDampTransitionVariables<f32>,
    pub eyes_translation: CylinderCoordinates3dSmoothDampTransitionVariables,
    pub eyes_roll: SmoothDampTransitionVariables<f32>,
}

// TODO transition state parameters

// state for "cylinder" behavior of player camera
pub struct PlayerCameraCylinderState {
    /// translation of origin, in world space.
    pub origin_translation: Vec3,

    /// longitudinal axis for cylindrical coordinates
    pub origin_up: Vec3,

    /// translation of eyes, with respect to origin.
    pub eyes_translation: CylindricalCoordinates,

    /// direction which is up for eyes
    pub eyes_up: Vec3,

    /// direction which is forward for eyes
    pub eyes_forward: Vec3,

    pub eyes_roll: f32,
}

/// current state for "cylinder" behavior for player camera.
#[derive(Component)]
pub struct PlayerCameraCylinderTransitionCurrentStateComponent {
    pub camera_state: PlayerCameraCylinderState,
}

/// desired state for "cylinder" behavior for player camera.
#[derive(Component)]
pub struct PlayerCameraCylinderTransitionDesiredStateComponent {
    pub camera_state: PlayerCameraCylinderState,
}

#[derive(Bundle)]
pub struct PlayerBundle {
    pub tag: PlayerTagComponent,
    pub camera_cylinder_transition_current_state:
        PlayerCameraCylinderTransitionCurrentStateComponent,
    pub camera_cylinder_transition_desired_state:
        PlayerCameraCylinderTransitionDesiredStateComponent,
    pub camera_cylinder_transition_variables: PlayerCameraCylinderTransitionVariablesComponent,
}

// REGION update transition desired state

/// set desired translation for player camera origin.
pub fn set_player_camera_cylinder_desired_state_origin_translation_using_character_system(
    mut player_query: Query<
        (&mut PlayerCameraCylinderTransitionDesiredStateComponent,),
        (With<PlayerTagComponent>,),
    >,
    character_query: Query<(&Transform,), (With<CharacterTagComponent>,)>,
) {
    let character = character_query.single();
    let mut player = player_query.single_mut();

    player.0.camera_state.origin_translation = character.0.translation;
}

/// set desired up for player camera origin.
pub fn set_player_camera_cylinder_desired_state_origin_rotation_using_character_system(
    mut player_query: Query<
        (&mut PlayerCameraCylinderTransitionDesiredStateComponent,),
        (With<PlayerTagComponent>,),
    >,
    character_query: Query<(&Transform,), With<CharacterTagComponent>>,
) {
    let character_result = character_query.get_single();
    let mut player = player_query.single_mut();

    if character_result.is_err() {
        return;
    }

    let character = character_result.unwrap();

    // TODO deadzones on the verticals

    player.0.camera_state.origin_up = *character.0.local_y();
}

/// set desired up for player camera origin.
pub fn set_player_camera_cylinder_desired_state_eyes_up_using_character_system(
    mut player_query: Query<
        (&mut PlayerCameraCylinderTransitionDesiredStateComponent,),
        (With<PlayerTagComponent>,),
    >,
    character_query: Query<(&Transform,), With<CharacterTagComponent>>,
) {
    let character_result = character_query.get_single();
    let mut player = player_query.single_mut();

    if character_result.is_err() {
        return;
    }

    let character = character_result.unwrap();

    // TODO deadzones on the verticals

    player.0.camera_state.eyes_up = *character.0.local_y();
}

/// set desired roll for player camera eyes.
pub fn set_player_camera_cylinder_eyes_desired_state_roll_using_input_system(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut player_query: Query<
        (&mut PlayerCameraCylinderTransitionDesiredStateComponent,),
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

/// set desired height and lookat for player camera eyes.
pub fn set_player_camera_cylinder_desired_state_eyes_translation_and_forward_using_input_system(
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut player_query: Query<
        (
            &mut Transform,
            &mut PlayerCameraCylinderTransitionDesiredStateComponent,
        ),
        (With<PlayerTagComponent>, Without<CharacterTagComponent>),
    >,
) {
    let mut player = player_query.single_mut();

    let mut mouse_move_input = Vec2::ZERO;
    for mouse_event in mouse_motion_events.read() {
        mouse_move_input.x += mouse_event.delta.x * 0.001;
        mouse_move_input.y += mouse_event.delta.y * 0.001;
    }

    let mut mouse_wheel_input: f32 = 0.0;
    for mouse_event in mouse_wheel_events.read() {
        mouse_wheel_input += mouse_event.y * 0.1;
    }

    // translation
    player.1.camera_state.eyes_translation.distance -= mouse_wheel_input;
    player.1.camera_state.eyes_translation.rotation =
        (player.1.camera_state.eyes_translation.rotation + mouse_move_input.x) % (PI * 2.0);
    player.1.camera_state.eyes_translation.height -= mouse_move_input.y * 0.5;

    // forward
    let next_camera_forward_rotation =
        Quat::from_axis_angle(Vec3::Y, player.1.camera_state.eyes_translation.rotation);
    let mut next_camera_forward = Quat::mul_vec3(next_camera_forward_rotation, Vec3::NEG_X);
    //next_camera_forward.y -= mouse_move_input.y;
    player.1.camera_state.eyes_forward = next_camera_forward;
}

pub fn set_player_camera_cylinder_desired_state_eyes_roll_on_mouse_input_system(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut player_query: Query<
        (&mut PlayerCameraCylinderTransitionDesiredStateComponent,),
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

pub fn transition_player_camera_cylinder_origin_translation_system(
    time: Res<Time>,
    mut player_query: Query<
        (
            &PlayerCameraCylinderTransitionDesiredStateComponent,
            &mut PlayerCameraCylinderTransitionVariablesComponent,
            &mut PlayerCameraCylinderTransitionCurrentStateComponent,
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

pub fn transition_player_camera_cylinder_origin_rotation_system(
    time: Res<Time>,
    mut player_query: Query<
        (
            &PlayerCameraCylinderTransitionDesiredStateComponent,
            &mut PlayerCameraCylinderTransitionCurrentStateComponent,
        ),
        (With<PlayerTagComponent>,),
    >,
) {
    for (player_camera_desired_state, mut player_camera_current_state) in &mut player_query {
        // TODO use smooth_damp
        let next_origin_up = Vec3::slerp(
            &player_camera_current_state.camera_state.origin_up,
            player_camera_desired_state.camera_state.origin_up,
            time.delta().as_secs_f32() * CAMERA_TRANSITON_SPEED,
        );

        player_camera_current_state.camera_state.origin_up = next_origin_up;
    }
}

/// transition player camera eyes current state towards desired state.
pub fn transition_player_camera_cylinder_eyes_rotation_system(
    time: Res<Time>,
    mut player_query: Query<
        (
            &PlayerCameraCylinderTransitionDesiredStateComponent,
            &mut PlayerCameraCylinderTransitionCurrentStateComponent,
        ),
        (With<PlayerTagComponent>,),
    >,
) {
    for (player_camera_desired_state, mut player_camera_current_state) in &mut player_query {
        let next_rotation = f32::lerp_angle(
            player_camera_current_state
                .camera_state
                .eyes_translation
                .rotation,
            player_camera_desired_state
                .camera_state
                .eyes_translation
                .rotation,
            time.delta().as_secs_f32(),
        );

        println!(
            "transition_player_camera_cylinder_eyes_rotation_system {}",
            next_rotation
        );
        player_camera_current_state
            .camera_state
            .eyes_translation
            .rotation = next_rotation;
    }
}

/// transition player camera eyes current state towards desired state.
pub fn transition_player_camera_cylinder_eyes_height_system(
    time: Res<Time>,
    mut player_query: Query<
        (
            &PlayerCameraCylinderTransitionDesiredStateComponent,
            &mut PlayerCameraCylinderTransitionVariablesComponent,
            &mut PlayerCameraCylinderTransitionCurrentStateComponent,
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
pub fn transition_player_camera_cylinder_eyes_distance_system(
    time: Res<Time>,
    mut player_query: Query<
        (
            &PlayerCameraCylinderTransitionDesiredStateComponent,
            &mut PlayerCameraCylinderTransitionVariablesComponent,
            &mut PlayerCameraCylinderTransitionCurrentStateComponent,
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
pub fn transition_player_camera_cylinder_eyes_up_system(
    time: Res<Time>,
    mut player_query: Query<
        (
            &PlayerCameraCylinderTransitionDesiredStateComponent,
            &mut PlayerCameraCylinderTransitionCurrentStateComponent,
        ),
        With<PlayerTagComponent>,
    >,
) {
    for (player_camera_desired_state, mut player_camera_current_state) in &mut player_query {
        // TODO use smoothdamp instead of slerp
        let next_eyes_up = Vec3::slerp(
            &player_camera_current_state.camera_state.eyes_up,
            player_camera_desired_state.camera_state.eyes_up,
            time.delta().as_secs_f32() * CAMERA_TRANSITON_SPEED,
        );

        player_camera_current_state.camera_state.eyes_up = next_eyes_up;
    }
}

/// transition player camera eyes current state towards desired state.
pub fn transition_player_camera_cylinder_eyes_roll_system(
    time: Res<Time>,
    mut player_query: Query<
        (
            &PlayerCameraCylinderTransitionDesiredStateComponent,
            &mut PlayerCameraCylinderTransitionVariablesComponent,
            &mut PlayerCameraCylinderTransitionCurrentStateComponent,
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

/// transition player camera eyes forward current state towards desired state.
pub fn transition_player_camera_cylinder_eyes_forward_system(
    time: Res<Time>,
    mut player_query: Query<
        (
            &PlayerCameraCylinderTransitionDesiredStateComponent,
            &mut PlayerCameraCylinderTransitionCurrentStateComponent,
        ),
        (With<PlayerTagComponent>,),
    >,
) {
    for (player_camera_desired_state, mut player_camera_current_state) in &mut player_query {
        let next_camera_eyes_forward = Vec3::slerp(
            &player_camera_current_state.camera_state.eyes_forward,
            player_camera_desired_state.camera_state.eyes_forward,
            time.delta().as_secs_f32(),
        );

        player_camera_current_state.camera_state.eyes_forward = next_camera_eyes_forward;
    }
}

// REGIONEND

// REGION update state (writeback to core components)

/// apply player camera eyes transform using current state system.
pub fn apply_player_camera_cylinder_transform_using_current_state_system(
    mut player_query: Query<
        (
            &PlayerCameraCylinderTransitionCurrentStateComponent,
            &mut Transform,
        ),
        (With<PlayerTagComponent>, Without<CharacterTagComponent>),
    >,
) {
    let mut player = player_query.single_mut();
    let next_origin_translation: Vec3 = player.0.camera_state.origin_translation.clone();
    let eyes_local_translation = Vec3::from_cylindrical(&player.0.camera_state.eyes_translation);

    let origin_rotation = Quat::from_rotation_arc(Vec3::Y, player.0.camera_state.eyes_up);

    player.1.translation =
        next_origin_translation + Quat::mul_vec3(origin_rotation, eyes_local_translation);

    let next_transform = Transform::looking_to(
        *player.1,
        player.0.camera_state.eyes_forward,
        player.0.camera_state.eyes_up,
    );

    *player.1 = next_transform;
    player.1.rotate_local_z(player.0.camera_state.eyes_roll);
}

// REGIONEND

pub fn draw_player_camera_cylinder_forward_gizmos_system(
    mut gizmos: Gizmos,
    player_query: Query<
        (
            &PlayerCameraCylinderTransitionCurrentStateComponent,
            &PlayerCameraCylinderTransitionDesiredStateComponent,
        ),
        With<PlayerTagComponent>,
    >,
) {
    let player = player_query.single();

    gizmos.arrow(
        player.1.camera_state.origin_translation,
        player.1.camera_state.eyes_forward,
        Color::YELLOW,
    );
}

pub fn draw_player_camera_cylinder_gizmos_system(
    mut gizmos: Gizmos,
    player_query: Query<
        (&PlayerCameraCylinderTransitionCurrentStateComponent,),
        With<PlayerTagComponent>,
    >,
) {
    let player = player_query.single();

    let player_camera_origin_current_translation = player.0.camera_state.origin_translation;

    gizmos.circle(
        player_camera_origin_current_translation,
        Direction3d::new_unchecked(player.0.camera_state.origin_up),
        player.0.camera_state.eyes_translation.distance,
        Color::WHITE,
    );

    // up axis
    gizmos.line(
        player_camera_origin_current_translation,
        player_camera_origin_current_translation + player.0.camera_state.origin_up * 5.0,
        Color::WHITE,
    );
}
