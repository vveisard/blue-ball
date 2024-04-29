use crate::{
    character::{
        CharacterPlayerInputComponent, CharacterRotationFromGlobalToCharacterParametersComponent,
    },
    math::{
        CylinderCoordinates3d, CylinderCoordinates3dSmoothDampTransitionVariables,
        SmoothDampTransitionVariables,
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
    math::{Quat, Vec2, Vec3},
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
pub struct PlayerCameraTransitionStateVariablesComponent {
    pub transition_cylinder_coordinates: CylinderCoordinates3dSmoothDampTransitionVariables,
    pub focus: SmoothDampTransitionVariables<Vec3>,
    pub roll: SmoothDampTransitionVariables<f32>,
}

// TODO transition state parameters

pub struct PlayerCameraState {
    /// translation with respect to the character
    pub local_cylinder_coordinates: CylinderCoordinates3d,
    // point to lookat
    pub focus: Vec3,

    pub roll: f32,
}

/// current state of player camera.
#[derive(Component)]
pub struct PlayerCameraCurrentStateComponent {
    pub camera_state: PlayerCameraState,
}

/// desired state of player camera.
#[derive(Component)]
pub struct PlayerCameraDesiredStateComponent {
    pub camera_state: PlayerCameraState,
}

#[derive(Bundle)]
pub struct PlayerBundle {
    pub tag: PlayerTagComponent,
    pub camera_current_state: PlayerCameraCurrentStateComponent,
    pub camera_desired_state: PlayerCameraDesiredStateComponent,
    pub camera_transition_variables: PlayerCameraTransitionStateVariablesComponent,
}

pub fn update_player_camera_state_roll_using_input_system(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut player_query: Query<(&mut PlayerCameraDesiredStateComponent,), With<PlayerTagComponent>>,
) {
    let mut player = player_query.single_mut();

    if mouse_button_input.pressed(MouseButton::Left) {
        player.0.camera_state.roll -= 0.01
    }

    if mouse_button_input.pressed(MouseButton::Right) {
        player.0.camera_state.roll += 0.01
    }
}

pub fn update_player_camera_desired_state_coordinates_using_input_system(
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut player_query: Query<
        (&mut Transform, &mut PlayerCameraDesiredStateComponent),
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

    player.1.camera_state.local_cylinder_coordinates.distance -= zoom_input;

    player.1.camera_state.local_cylinder_coordinates.rotation += input.x;
    player.1.camera_state.focus.y -= input.y;
    player.1.camera_state.local_cylinder_coordinates.height -= input.y * 0.5;
}

pub fn transition_player_camera_current_state_rotation_system(
    time: Res<Time>,
    mut player_camera_query: Query<
        (
            &PlayerCameraDesiredStateComponent,
            &mut PlayerCameraTransitionStateVariablesComponent,
            &mut PlayerCameraCurrentStateComponent,
        ),
        (With<PlayerTagComponent>,),
    >,
) {
    for (
        player_camera_desired_state,
        mut player_camera_transition_state_variables,
        mut player_camera_current_state,
    ) in &mut player_camera_query
    {
        let smooth_damp_result = f32::smooth_damp(
            player_camera_current_state
                .camera_state
                .local_cylinder_coordinates
                .rotation,
            player_camera_desired_state
                .camera_state
                .local_cylinder_coordinates
                .rotation,
            player_camera_transition_state_variables
                .transition_cylinder_coordinates
                .rotation
                .velocity,
            0.1,
            f32::INFINITY,
            time.delta().as_secs_f32(),
        );

        player_camera_current_state
            .camera_state
            .local_cylinder_coordinates
            .rotation = smooth_damp_result.0;

        player_camera_transition_state_variables
            .transition_cylinder_coordinates
            .rotation
            .velocity = smooth_damp_result.1;
    }
}

pub fn transition_player_camera_state_height_system(
    time: Res<Time>,
    mut player_camera_query: Query<
        (
            &PlayerCameraDesiredStateComponent,
            &mut PlayerCameraTransitionStateVariablesComponent,
            &mut PlayerCameraCurrentStateComponent,
        ),
        (With<PlayerTagComponent>,),
    >,
) {
    for (
        player_camera_desired_state,
        mut player_camera_transition_state_variables,
        mut player_camera_current_state,
    ) in &mut player_camera_query
    {
        let smooth_damp_result = f32::smooth_damp(
            player_camera_current_state
                .camera_state
                .local_cylinder_coordinates
                .height,
            player_camera_desired_state
                .camera_state
                .local_cylinder_coordinates
                .height,
            player_camera_transition_state_variables
                .transition_cylinder_coordinates
                .height
                .velocity,
            0.1,
            f32::INFINITY,
            time.delta().as_secs_f32(),
        );

        player_camera_current_state
            .camera_state
            .local_cylinder_coordinates
            .height = smooth_damp_result.0;

        player_camera_transition_state_variables
            .transition_cylinder_coordinates
            .height
            .velocity = smooth_damp_result.1;
    }
}

pub fn transition_player_camera_state_distance_system(
    time: Res<Time>,
    mut player_camera_query: Query<
        (
            &PlayerCameraDesiredStateComponent,
            &mut PlayerCameraTransitionStateVariablesComponent,
            &mut PlayerCameraCurrentStateComponent,
        ),
        (With<PlayerTagComponent>,),
    >,
) {
    for (
        player_camera_desired_state,
        mut player_camera_transition_state_variables,
        mut player_camera_current_state,
    ) in &mut player_camera_query
    {
        let smooth_damp_result = f32::smooth_damp(
            player_camera_current_state
                .camera_state
                .local_cylinder_coordinates
                .distance,
            player_camera_desired_state
                .camera_state
                .local_cylinder_coordinates
                .distance,
            player_camera_transition_state_variables
                .transition_cylinder_coordinates
                .distance
                .velocity,
            0.1,
            f32::INFINITY,
            time.delta().as_secs_f32(),
        );

        player_camera_current_state
            .camera_state
            .local_cylinder_coordinates
            .distance = smooth_damp_result.0;

        player_camera_transition_state_variables
            .transition_cylinder_coordinates
            .distance
            .velocity = smooth_damp_result.1;
    }
}

pub fn transition_player_camera_state_roll_system(
    time: Res<Time>,
    mut player_camera_query: Query<
        (
            &PlayerCameraDesiredStateComponent,
            &mut PlayerCameraTransitionStateVariablesComponent,
            &mut PlayerCameraCurrentStateComponent,
        ),
        (With<PlayerTagComponent>,),
    >,
) {
    for (
        player_camera_desired_state,
        mut player_camera_transition_state_variables,
        mut player_camera_current_state,
    ) in &mut player_camera_query
    {
        let smooth_damp_result = f32::smooth_damp(
            player_camera_current_state.camera_state.roll,
            player_camera_desired_state.camera_state.roll,
            player_camera_transition_state_variables.roll.velocity,
            0.1,
            f32::INFINITY,
            time.delta().as_secs_f32(),
        );

        player_camera_current_state.camera_state.roll = smooth_damp_result.0;

        player_camera_transition_state_variables.roll.velocity = smooth_damp_result.1;
    }
}

pub fn transition_player_camera_state_focus_system(
    time: Res<Time>,
    mut player_camera_query: Query<
        (
            &PlayerCameraDesiredStateComponent,
            &mut PlayerCameraTransitionStateVariablesComponent,
            &mut PlayerCameraCurrentStateComponent,
        ),
        (With<PlayerTagComponent>,),
    >,
) {
    for (
        player_camera_desired_state,
        mut player_camera_transition_state_variables,
        mut player_camera_current_state,
    ) in &mut player_camera_query
    {
        let smooth_damp_result = Vec3::smooth_damp(
            player_camera_current_state.camera_state.focus,
            player_camera_desired_state.camera_state.focus,
            player_camera_transition_state_variables.focus.velocity,
            0.1,
            f32::INFINITY,
            time.delta().as_secs_f32(),
        );

        player_camera_current_state.camera_state.focus = smooth_damp_result.0;

        player_camera_transition_state_variables.focus.velocity = smooth_damp_result.1;
    }
}

pub fn update_player_camera_transform_using_state_system(
    mut character_query: Query<
        (&Transform,),
        (With<CharacterTagComponent>, Without<PlayerTagComponent>),
    >,
    mut player_query: Query<
        (&mut Transform, &PlayerCameraCurrentStateComponent),
        (With<PlayerTagComponent>, Without<CharacterTagComponent>),
    >,
) {
    let character = character_query.single_mut();
    let mut player = player_query.single_mut();

    let relative_x_translation = player.1.camera_state.local_cylinder_coordinates.distance
        * f32::cos(player.1.camera_state.local_cylinder_coordinates.rotation);
    let relative_z_translation = player.1.camera_state.local_cylinder_coordinates.distance
        * f32::sin(player.1.camera_state.local_cylinder_coordinates.rotation);
    let relative_y_translation = player.1.camera_state.local_cylinder_coordinates.height;

    player.0.translation = character.0.translation
        + Vec3::new(
            relative_x_translation,
            relative_y_translation,
            relative_z_translation,
        );

    player.0.look_at(
        character.0.translation + player.1.camera_state.focus,
        Vec3::Y,
    );

    player.0.rotate_local_z(player.1.camera_state.roll);
}

pub fn reset_player_roll_on_mouse_input_system(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut player_query: Query<(&mut PlayerCameraDesiredStateComponent,), With<PlayerTagComponent>>,
) {
    if !mouse_button_input.just_pressed(MouseButton::Middle) {
        return;
    }

    player_query.single_mut().0.camera_state.roll = 0.0;
}

pub fn draw_player_camera_focus_gizmos_system(
    mut gizmos: Gizmos,
    player_query: Query<
        (
            &PlayerCameraCurrentStateComponent,
            &PlayerCameraDesiredStateComponent,
        ),
        With<PlayerTagComponent>,
    >,
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
    let player = player_query.single();

    gizmos.sphere(
        character.0.translation + player.0.camera_state.focus,
        Quat::IDENTITY,
        0.5,
        Color::WHITE,
    );

    gizmos.sphere(
        character.0.translation + player.1.camera_state.focus,
        Quat::IDENTITY,
        0.5,
        Color::YELLOW,
    );
}
