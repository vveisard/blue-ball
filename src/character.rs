use bevy::{
    ecs::{
        bundle::Bundle,
        component::Component,
        entity::Entity,
        query::{With, Without},
        system::{Commands, Query, Res},
    },
    hierarchy::Children,
    math::{Quat, Vec3},
    render::view::InheritedVisibility,
    transform::components::{GlobalTransform, Transform},
};
use bevy_rapier3d::{
    dynamics::Velocity,
    geometry::{Collider, CollisionGroups, Group},
    pipeline::QueryFilter,
    plugin::RapierContext,
};

#[derive(Component)]
pub struct CharacterTagComponent;

// tag component for body of a character
#[derive(Component)]
pub struct CharacterBodyTagComponent;

/// Character component.
#[derive(Component)]
pub struct CharacterRotationFromGlobalToCharacterParametersComponent {
    /// rotation from global space to character space
    pub rotation_from_camera_to_character_quat: Quat,
}

/// component with input from player for character.
#[derive(Component)]
pub struct CharacterPlayerInputComponent {
    /// movement input from camera space rotated onto the character's horizontal (xz) plane.
    /// "natural" transformation, as opposted to projection
    pub natural_movement_player_input: Vec3,

    pub do_activate_jump_input: bool,
}

/// variables for character movement on the stage
#[derive(Component)]
pub struct CharacterMovementVariablesComponent {
    /// character global horizontal movement, on their up direction.
    pub global_horizontal_velocity: Vec3,

    /// this has a y component when about to leave the stage
    pub local_vertical_velocity: f32,
}

/// component for stage state of a character.
#[derive(Component)]
pub struct CharacterStageComponent {
    pub stage_entity: Entity,
}

/// parameters for the "fall" phase of a character.
#[derive(Component)]
pub struct CharacterFallPhaseMovementParametersComponent {
    pub maximum_down_speed: f32,
    pub maximum_up_speed: f32,
    /// aka "gravity"
    pub down_acceleration: f32,
}

#[derive(Bundle)]
pub struct CharacterBundle {
    pub tag: CharacterTagComponent,
    pub global_transform: GlobalTransform,
    pub transform: Transform,
    pub inherited_visibility: InheritedVisibility,
    pub rotation_from_player_to_character:
        CharacterRotationFromGlobalToCharacterParametersComponent,
    pub player_input: CharacterPlayerInputComponent,
    pub fall_phase_movement_parameters: CharacterFallPhaseMovementParametersComponent,
    pub movement_variables: CharacterMovementVariablesComponent,
}

/// system to update movement body velocity of a character which is on the stage
pub fn update_character_movement_velocity_while_on_stage_system(
    mut character_query: Query<
        (
            &Transform,
            &CharacterPlayerInputComponent,
            &mut CharacterMovementVariablesComponent,
        ),
        (With<CharacterTagComponent>, With<CharacterStageComponent>),
    >,
) {
    let character_result = character_query.get_single_mut();

    if character_result.is_err() {
        return;
    }

    let mut character = character_result.unwrap();

    character.2.global_horizontal_velocity = character.1.natural_movement_player_input * 8.0;

    if character.1.do_activate_jump_input {
        character.2.local_vertical_velocity += 15.0;
    }
}

/// system to update movement velocity of a character which is in the air
pub fn update_character_movement_velocity_while_in_air_phase_system(
    mut character_query: Query<
        (
            &CharacterFallPhaseMovementParametersComponent,
            &mut CharacterMovementVariablesComponent,
        ),
        (
            With<CharacterTagComponent>,
            Without<CharacterStageComponent>,
        ),
    >,
) {
    let character_result = character_query.get_single_mut();

    if character_result.is_err() {
        return;
    }

    let mut character = character_result.unwrap();

    character.1.local_vertical_velocity = f32::clamp(
        character.1.local_vertical_velocity - character.0.down_acceleration,
        -character.0.maximum_down_speed,
        character.0.maximum_up_speed,
    );
}

/// system to update physics body velocity for a character using movement velocity
pub fn update_character_body_velocity_while_on_stage_using_movement_velocity_system(
    mut character_query: Query<
        (
            &Transform,
            &CharacterPlayerInputComponent,
            &CharacterRotationFromGlobalToCharacterParametersComponent,
            &CharacterMovementVariablesComponent,
            &mut Velocity,
        ),
        (With<CharacterTagComponent>, With<CharacterStageComponent>),
    >,
) {
    let character_result = character_query.get_single_mut();

    if character_result.is_err() {
        return;
    }

    let mut character = character_result.unwrap();

    let next_body_velocity = character.3.global_horizontal_velocity
        + character.0.up() * character.3.local_vertical_velocity;

    character.4.linvel = next_body_velocity;
}

/// system to update physics body velocity for a character using movement velocity
pub fn update_character_body_velocity_while_in_air_using_movement_velocity_system(
    mut character_query: Query<
        (
            &CharacterPlayerInputComponent,
            &CharacterRotationFromGlobalToCharacterParametersComponent,
            &CharacterMovementVariablesComponent,
            &mut Velocity,
        ),
        (
            With<CharacterTagComponent>,
            Without<CharacterStageComponent>,
        ),
    >,
) {
    let character_result = character_query.get_single_mut();

    if character_result.is_err() {
        return;
    }

    let mut character = character_result.unwrap();

    character.3.linvel =
        character.2.global_horizontal_velocity + Vec3::Y * character.2.local_vertical_velocity
}

pub fn update_character_on_stage_body_position_system(
    rapier_context: Res<RapierContext>,
    mut commands: Commands,
    mut character_query: Query<
        (Entity, &Children, &Velocity, &mut Transform),
        (With<CharacterTagComponent>, With<CharacterStageComponent>),
    >,
    character_body_query: Query<
        (&Transform, &GlobalTransform, &Collider),
        (
            With<CharacterBodyTagComponent>,
            Without<CharacterTagComponent>,
        ),
    >,
) {
    let character_result = character_query.get_single_mut();

    if character_result.is_err() {
        return;
    }

    let mut character = character_result.unwrap();
    let character_body: (&Transform, &GlobalTransform, &Collider) = character_body_query.single();
    let character_velocity = character.2;
    let character_hips_position = character_body.1.translation();
    let character_hips_down = character_body.1.down();
    let character_hips_height = character_body.0.translation.y;

    // TOOD consider changing logic to specially consider "current" stage collider

    // from hips
    if let Some((stage_entity, ray_intersection)) = rapier_context.cast_ray_and_get_normal(
        character_hips_position,
        character_hips_down,
        character_hips_height + 0.16,
        true,
        QueryFilter::new().groups(CollisionGroups::new(
            Group::from_bits(0b0100).unwrap(),
            Group::from_bits(0b0010).unwrap(),
        )),
    ) {
        println!(
            "moving on stage. {} {}",
            character_hips_position, character_hips_down
        );

        // snap to ground
        let rotation: Quat = Quat::from_rotation_arc(*character.3.up(), ray_intersection.normal);
        character.3.rotation *= rotation;
        character.3.translation = ray_intersection.point;

        commands
            .entity(character.0)
            .insert(CharacterStageComponent {
                stage_entity: stage_entity,
            });

        return;
    }

    // moving upwards, did not collide from hips
    // try feet
    if character_velocity.linvel.y > 0.0 {
        // TODO calculate exact length of raycast using trigommetry and maximum incline

        // from feet
        let character_feet_position =
            character_hips_position + character_hips_down * character_hips_height;
        let character_feet_snap_distance = 0.32;

        println!("moving downwards on stage. {}", character_feet_position);

        if let Some((stage_entity, ray_intersection)) = rapier_context.cast_ray_and_get_normal(
            character_feet_position,
            Vec3::NEG_Y,
            character_feet_snap_distance,
            true,
            QueryFilter::new().groups(CollisionGroups::new(
                Group::from_bits(0b0100).unwrap(),
                Group::from_bits(0b0010).unwrap(),
            )),
        ) {
            // snap to ground
            let rotation = Quat::from_rotation_arc(*character.3.up(), ray_intersection.normal);
            character.3.rotation *= rotation;
            character.3.translation = ray_intersection.point;

            commands
                .entity(character.0)
                .insert(CharacterStageComponent {
                    stage_entity: stage_entity,
                });

            return;
        }
    }

    // become airborne
    // TODO instead, orient to inverse of "gravity"
    character.3.rotation = Quat::IDENTITY;
    commands
        .entity(character.0)
        .remove::<CharacterStageComponent>();

    println!("leave");
}

pub fn update_character_on_stage_system(
    mut commands: Commands,
    mut character_query: Query<
        (
            Entity,
            &mut Transform,
            &mut CharacterMovementVariablesComponent,
        ),
        (With<CharacterTagComponent>, With<CharacterStageComponent>),
    >,
) {
    let character_result = character_query.get_single_mut();

    if character_result.is_err() {
        return;
    }

    let mut character = character_result.unwrap();

    let vertical_velocity = character.2.local_vertical_velocity;

    if vertical_velocity <= 0.0 {
        return;
    }

    let jump_velocity = character.1.up() * vertical_velocity;

    character.2.global_horizontal_velocity += jump_velocity;
    character.2.global_horizontal_velocity.y = 0.0;
    character.2.local_vertical_velocity = jump_velocity.y;
    character.1.rotation = Quat::IDENTITY;

    println!("jump");

    commands
        .entity(character.0)
        .remove::<CharacterStageComponent>();
}

pub fn update_character_in_air_body_position_system(
    rapier_context: Res<RapierContext>,
    mut commands: Commands,
    mut character_query: Query<
        (
            Entity,
            &Children,
            &mut Velocity,
            &mut CharacterMovementVariablesComponent,
            &mut Transform,
        ),
        (
            With<CharacterTagComponent>,
            Without<CharacterStageComponent>,
        ),
    >,
    character_body_query: Query<
        (&Transform, &GlobalTransform, &Collider),
        (
            With<CharacterBodyTagComponent>,
            Without<CharacterTagComponent>,
        ),
    >,
) {
    let character_result = character_query.get_single_mut();

    if character_result.is_err() {
        return;
    }

    let mut character = character_result.unwrap();

    let character_body: (&Transform, &GlobalTransform, &Collider) = character_body_query.single();
    let character_hips_position = character_body.1.translation();
    let character_hips_down = character_body.1.down();
    let character_hips_height = character_body.0.translation.y;

    // TOOD consider changing logic to specially consider "current" stage collider

    if character.2.linvel.y >= 0.0 {
        return;
    }

    println!("moving down in air");

    // from hips
    if let Some((stage_entity, ray_intersection)) = rapier_context.cast_ray_and_get_normal(
        character_hips_position,
        character_hips_down,
        character_hips_height + 0.16,
        true,
        QueryFilter::new().groups(CollisionGroups::new(
            Group::from_bits(0b0100).unwrap(),
            Group::from_bits(0b0010).unwrap(),
        )),
    ) {
        // snap to ground
        let rotation: Quat = Quat::from_rotation_arc(*character.4.up(), ray_intersection.normal);
        character.4.rotation *= rotation;
        character.4.translation = ray_intersection.point;
        character.2.linvel.y = 0.0;
        character.3.local_vertical_velocity = 0.0;

        commands
            .entity(character.0)
            .insert(CharacterStageComponent {
                stage_entity: stage_entity,
            });

        println!("land {}", ray_intersection.point);

        return;
    }
}
