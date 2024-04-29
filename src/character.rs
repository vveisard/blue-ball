use bevy::{
    ecs::{
        bundle::Bundle,
        component::Component,
        query::{With, Without},
        system::{Query, Res},
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
    /// player input vector in global space
    /// ie, input transformed from local space to global space
    pub global_input: Vec3,
}

#[derive(Component)]
pub struct CharacterVelocityComponent {
    pub global_velocity: Vec3,
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
    pub velocity: CharacterVelocityComponent,
}

pub fn update_character_velocity_using_input_system(
    mut character_query: Query<
        (
            &CharacterPlayerInputComponent,
            &mut CharacterVelocityComponent,
            &mut Velocity,
        ),
        With<CharacterTagComponent>,
    >,
) {
    let mut character = character_query.single_mut();

    character.1.global_velocity = character.0.global_input * 8.0;
    character.2.linvel = character.0.global_input * 8.0;
}

pub fn update_character_rigidbody_position_system(
    rapier_context: Res<RapierContext>,
    mut character_query: Query<
        (&CharacterVelocityComponent, &Children, &mut Transform),
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
    let character_body_position = character_body.1.translation();
    let character_snap_direction = character_body.1.down();
    let character_snap_distance = 1.0 + 0.5; // leg length + offset

    if let Some((_, ray_intersection)) = rapier_context.cast_ray_and_get_normal(
        character_body_position,
        character_snap_direction,
        character_snap_distance,
        true,
        QueryFilter::new().groups(CollisionGroups::new(
            Group::from_bits(0b0100).unwrap(),
            Group::from_bits(0b0010).unwrap(),
        )),
    ) {
        // TOOD validate angle difference is not too steep

        // snap to ground
        let rotation = Quat::from_rotation_arc(*character.2.up(), ray_intersection.normal);
        character.2.rotation *= rotation;
        character.2.translation = ray_intersection.point;
    } else {
        // become airborne
        // TODO instead, orient to inverse of "gravity"
        character.2.rotation = Quat::IDENTITY;
    }
}
