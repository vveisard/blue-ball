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
            &CharacterRotationFromGlobalToCharacterParametersComponent,
            &mut CharacterVelocityComponent,
        ),
        With<CharacterTagComponent>,
    >,
) {
    let mut character = character_query.single_mut();

    character.2.global_velocity = character.0.global_input * 0.1;
}

pub fn update_character_rigidbody_position_using_input_system(
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
    let character_body = character_body_query.single();

    let speed = character.0.global_velocity.length();

    if speed <= 0.0 {
        return;
    }

    let character_body_position = character_body.1.translation();

    if let Some((entity, hit)) = rapier_context.cast_shape(
        character_body_position,
        Quat::IDENTITY,
        character.0.global_velocity,
        character_body.2,
        speed,
        false,
        QueryFilter::new().groups(CollisionGroups::new(
            Group::from_bits(0b0100).unwrap(),
            Group::from_bits(0b0010).unwrap(),
        )),
    ) {
        // The first collider hit has the entity `entity`. The `hit` is a
        // structure containing details about the hit configuration.
        println!(
            "CAN'T PROCEED. Hit the entity {:?} with the configuration: {:?}",
            entity, hit
        );

        // TODO slide:
        // - consume velocity towards hit direction
        // - translate using remaining velocity
    } else {
        character.2.translation += character.0.global_velocity;
    }
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
    let character_body_down = character_body.1.down();
    let character_speed = 1.0 + 0.1; // height + skin

    if let Some((_, ray_intersection)) = rapier_context.cast_ray_and_get_normal(
        character_body_position,
        character_body_down,
        character_speed,
        true,
        QueryFilter::new().groups(CollisionGroups::new(
            Group::from_bits(0b0100).unwrap(),
            Group::from_bits(0b0010).unwrap(),
        )),
    ) {
        character.2.translation = ray_intersection.point;
        // TODO update rotation
    } else {
        // here, become airborne
    }
}
