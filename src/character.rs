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
    pub rotation_from_global_to_character_quat: Quat,
}

/// component with input from player for character.
#[derive(Component)]
pub struct CharacterPlayerInputComponent {
    /// player input in global space, transformed to player
    /// ie, input transformed from local space to global space
    pub global_character_input: Vec3,
}

/// tag parameter for on stage state of a character.
#[derive(Component)]
pub struct CharacterIsOnStageComponent {
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
}

pub fn update_character_velocity_while_on_stage_system(
    mut character_query: Query<
        (&CharacterPlayerInputComponent, &mut Velocity),
        (
            With<CharacterTagComponent>,
            With<CharacterIsOnStageComponent>,
        ),
    >,
) {
    let character_result = character_query.get_single_mut();

    if character_result.is_err() {
        return;
    }

    let mut character = character_result.unwrap();

    character.1.linvel = character.0.global_character_input * 8.0;
}

pub fn update_character_velocity_while_in_air_phase_system(
    mut character_query: Query<
        (
            &CharacterFallPhaseMovementParametersComponent,
            &mut Velocity,
        ),
        (
            With<CharacterTagComponent>,
            Without<CharacterIsOnStageComponent>,
        ),
    >,
) {
    let character_result = character_query.get_single_mut();

    if character_result.is_err() {
        return;
    }

    let mut character = character_result.unwrap();

    character.1.linvel.y = f32::clamp(
        character.1.linvel.y - character.0.down_acceleration,
        -character.0.maximum_down_speed,
        character.0.maximum_up_speed,
    );
}

pub fn update_character_rigidbody_position_system(
    rapier_context: Res<RapierContext>,
    mut commands: Commands,
    mut character_query: Query<
        (Entity, &Children, &Velocity, &mut Transform),
        With<CharacterTagComponent>,
    >,
    character_body_query: Query<
        (&Transform, &GlobalTransform, &Collider),
        (
            With<CharacterBodyTagComponent>,
            Without<CharacterTagComponent>,
        ),
    >,
) {
    let mut character = character_query.single_mut();
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
        // snap to ground
        let rotation: Quat = Quat::from_rotation_arc(*character.3.up(), ray_intersection.normal);
        character.3.rotation *= rotation;
        character.3.translation = ray_intersection.point;

        commands
            .entity(character.0)
            .insert(CharacterIsOnStageComponent {
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

        println!(
            "moving upwards. {} {}",
            character_feet_position, character_feet_snap_distance
        );

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
                .insert(CharacterIsOnStageComponent {
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
        .remove::<CharacterIsOnStageComponent>();

    println!("airborne");
}
