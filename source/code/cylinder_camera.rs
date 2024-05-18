use bevy::{
    ecs::{
        bundle::Bundle,
        component::Component,
        entity::Entity,
        event::EventReader,
        query::With,
        system::{Query, Res},
    },
    hierarchy::Parent,
    input::mouse::{
        MouseMotion, MouseWheel,
    },
    math::{Quat, Vec2, Vec3},
    time::Time,
    transform::components::{
        GlobalTransform, Transform,
    },
};

use crate::math::{
    CylindricalCoordinates,
    FromCylindrical,
};

/// Tag component for "camera body" entity.
#[derive(Component)]
pub struct CameraBodyTagComponent;

/// bundle for "cylinder camera body" entity.
#[derive(Bundle)]
pub struct CylinderCameraBodyBundle {
    pub tag: CameraBodyTagComponent,
    pub desired_transform_variables: DesiredTransformVariablesComponent,
    pub observed_entity_variables: ObservedEntityVariablesComponent,
    pub set_desired_transform_translation_to_observed_entity_transform_translation:
        SetDesiredTransformTranslationToObservedEntityTransformTranslationBehaviorComponent,
    pub set_desired_transform_local_up_to_observed_entity_local_up:
        SetDesiredTransformRotationToObservedEntityLocalUpBehaviorComponent,
}

/// Tag component for "camera eyes" entity.
#[derive(Component)]
pub struct CameraEyesTagComponent;

/// bundle for "cylinder camera eyes" entity
#[derive(Bundle)]
pub struct CylinderCameraEyesBundle {
    pub tag: CameraEyesTagComponent,
    pub observed_entity: ObservedEntityVariablesComponent,
    pub desired_transform_variables: DesiredTransformVariablesComponent,
    pub lookat_variables: LookatVariablesComponent,
    pub lookat_offset_variables: LookatOffsetVariablesComponent,
    pub cylinder_coordindates_for_desired_transform_translation_variables:
        CylinderCoordinatesForDesiredTransformTranslationVariablesComponent,
            pub set_lookat_position_to_observed_entity_transform_translation_with_offset_behavior:
        SetLookatPositionToObservedEntityTransformTranslationWithOffsetBehaviorComponent,
    pub set_cylinder_coordinate_for_desired_transform_translation_angle_using_input_behavior:
        SetCylinderCoordinateForDesiredTransformTranslationUsingInputBehaviorComponent,
    pub set_lookat_offset_using_input_behavior: SetLookatOffsetUsingInputBehaviorComponent
}

// REGION variables component

/// component with variables for desired transform.
/// ie, the [Transform] of this entity will be transitioned to [DesiredTransformVariablesComponent].
#[derive(Component)]
pub struct DesiredTransformVariablesComponent
{
    pub desired_transform: Transform,
}

/// component with variables for an observed entity.
/// ie, this entity is observing other.
#[derive(Component)]
pub struct ObservedEntityVariablesComponent
{
    pub entity: Entity,
}

/// component with variables for "lookat".
#[derive(Component)]
pub struct LookatVariablesComponent {
    pub position_relative_to_parent:
        Vec3,
    pub up: Vec3,
}

/// component with variables for offset from a base "lookat".
#[derive(Component)]
pub struct LookatOffsetVariablesComponent
{
    pub translation_wrt_parent: Vec3,
}

/// component with cylinder coordinates for [DesiredTransformVariablesComponent].
#[derive(Component)]
pub struct CylinderCoordinatesForDesiredTransformTranslationVariablesComponent
{
    pub cylinder_coordindates:
        CylindricalCoordinates,
}

// REGIONEND

// REGION behavior component

/// component for [set_desired_transform_translation_to_observed_entiy_transform_translation_behavior_system].
#[derive(Component)]
pub struct SetDesiredTransformTranslationToObservedEntityTransformTranslationBehaviorComponent;

/// component for [set_desired_transform_rotation_to_observed_entity_local_up_behavior_system].
#[derive(Component)]
pub struct SetDesiredTransformRotationToObservedEntityLocalUpBehaviorComponent;

/// component for [set_lookat_position_to_observed_entity_transform_translation_with_offset_behavior_system].
#[derive(Component)]
pub struct SetLookatPositionToObservedEntityTransformTranslationWithOffsetBehaviorComponent;

/// component for [set_cylinder_coordinates_for_desired_transform_translation_using_input_system].
#[derive(Component)]
pub struct SetCylinderCoordinateForDesiredTransformTranslationUsingInputBehaviorComponent;

/// behavior component to update [LookatOffsetVariablesComponent] using input.
#[derive(Component)]
pub struct SetLookatOffsetUsingInputBehaviorComponent;

// REGIONEND

// REGION transition system

/// transition [Transform] using [DesiredTransformVariablesComponent].
pub fn transition_desired_transform_to_transform_system(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &DesiredTransformVariablesComponent)>,
) {
    for (
        mut transform,
        desired_transform_variables,
    ) in query.iter_mut()
    {
        let next_position =
            desired_transform_variables
                .desired_transform
                .translation;
        let next_rotation = Quat::slerp(
            transform.rotation,
            desired_transform_variables
                .desired_transform
                .rotation,
            time.delta().as_secs_f32()
                * 3.33,
        );

        transform.translation =
            next_position;
        transform.rotation =
            next_rotation;
    }
}

// ENDREGION

// REGION apply system

/// update [Transform] using [LookatVariablesComponent].
pub fn apply_lookat_to_transform_system(
    mut query: Query<(
        &mut Transform,
        &LookatVariablesComponent,
    )>,
) {
    for (
        mut transform,
        lookat_variables,
    ) in query.iter_mut()
    {
        transform.look_at(
            lookat_variables.position_relative_to_parent,
            lookat_variables.up,
        );
    }
}

/// update [DesiredTransformVariablesComponent] using [CylinderCoordinatesForDesiredTransformTranslationVariablesComponent].
pub fn apply_desired_transform_using_cylinder_coordinates_system(
    mut query: Query<(
        &mut DesiredTransformVariablesComponent,
        &CylinderCoordinatesForDesiredTransformTranslationVariablesComponent,
    )>,
) {
    for (
        mut desired_transform_variables,
        cylinder_coordinates_for_desired_transform,
    ) in query.iter_mut()
    {
        let next = Vec3::from_cylindrical(
            &cylinder_coordinates_for_desired_transform.cylinder_coordindates,
        );

        desired_transform_variables
            .desired_transform
            .translation = next;
    }
}

// REGIONEND

// REGION behavior system

/// set [DesiredTransformVariablesComponent] using [ObservedEntityVariablesComponent].
pub fn set_desired_transform_translation_to_observed_entiy_transform_translation_behavior_system(
    mut query: Query<
        (
            &mut DesiredTransformVariablesComponent,
            &ObservedEntityVariablesComponent,
        ),
        With<SetDesiredTransformTranslationToObservedEntityTransformTranslationBehaviorComponent>,
    >,
    observed_query: Query<
        (&Transform,),
    >,
) {
    for (
        mut desired_transform,
        &ObservedEntityVariablesComponent {
            entity: observed_entity,
        },
    ) in query.iter_mut()
    {
        let observed_entity_transform = observed_query
            .get(observed_entity)
            .expect("Observed entity despawned!");

        desired_transform.desired_transform.translation = observed_entity_transform.0.translation;
    }
}

/// set [DesiredTransformVariablesComponent] using [ObservedEntityVariablesComponent].
pub fn set_desired_transform_rotation_to_observed_entity_local_up_behavior_system(
    mut query: Query<
        (
            &mut DesiredTransformVariablesComponent,
            &ObservedEntityVariablesComponent,
        ),
        With<SetDesiredTransformRotationToObservedEntityLocalUpBehaviorComponent>,
    >,
    observed_query: Query<
        (&Transform,),
    >,
) {
    for (
        mut desired_transform,
        &ObservedEntityVariablesComponent {
            entity: observed_entity,
        },
    ) in query.iter_mut()
    {
        let observed_entity_transform = observed_query
            .get(observed_entity)
            .expect("Observed entity despawned!");

        let rotation = Quat::from_rotation_arc(Vec3::Y, *observed_entity_transform.0.local_y());

        desired_transform.desired_transform.rotation = rotation;
    }
}

/// set [LookatVariablesComponent] on [SetLookatPositionToObservedEntityTransformTranslationWithOffsetBehaviorComponent] using [LookatOffsetVariablesComponent].
pub fn set_lookat_position_to_observed_entity_transform_translation_with_offset_behavior_system(
    mut query: Query<
        (&mut LookatVariablesComponent, &LookatOffsetVariablesComponent, &ObservedEntityVariablesComponent, &Parent),
        With<SetLookatPositionToObservedEntityTransformTranslationWithOffsetBehaviorComponent>,
    >,
    observed_query: Query<(
        &GlobalTransform,
    )>,
) {
    for (
      mut lookat_variables,  lookat_offset_variables,       &ObservedEntityVariablesComponent {
            entity: observed_entity,
        },
        parent
      ) in
        query.iter_mut()
    {
        let parent_entity = observed_query.get(**parent).expect("Parent entity desspawned!");

        let observed_entity = observed_query
            .get(observed_entity)
            .expect("Observed entity despawned!");

        let translation_wrt_parent  = parent_entity.0.affine().inverse().transform_point3(observed_entity.0.translation());

        lookat_variables.position_relative_to_parent = translation_wrt_parent + lookat_offset_variables.translation_wrt_parent;
    }
}

/// set [CylinderCoordinatesForDesiredTransformTranslationVariablesComponent] on [SetCylinderCoordinateForDesiredTransformTranslationUsingInputBehaviorComponent].
pub fn set_cylinder_coordinates_for_desired_transform_translation_using_input_system(
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut mouse_wheel_events: EventReader<
        MouseWheel,
    >,
    mut query: Query<
        (&mut CylinderCoordinatesForDesiredTransformTranslationVariablesComponent,),
        With<SetCylinderCoordinateForDesiredTransformTranslationUsingInputBehaviorComponent>,
    >,
) {
    let mut input = Vec2::ZERO;
    for mouse_event in
        mouse_motion_events.read()
    {
        input.x +=
            mouse_event.delta.x * 0.001;
        input.y +=
            mouse_event.delta.y * 0.001;
    }

    let mut zoom_input: f32 = 0.0;
    for mouse_event in
        mouse_wheel_events.read()
    {
        zoom_input +=
            mouse_event.y * 0.1;
    }

    for mut
    desired_cylinder_coordinates_for_transform in
        query.iter_mut()
    {
        desired_cylinder_coordinates_for_transform
            .0
            .cylinder_coordindates
            .distance -= zoom_input;

        desired_cylinder_coordinates_for_transform
            .0
            .cylinder_coordindates
            .rotation += input.x;
        desired_cylinder_coordinates_for_transform
            .0
            .cylinder_coordindates
            .height -= input.y * 0.5;
    }
}

/// set [LookatOffsetVariablesComponent] on [SetLookatOffsetUsingInputBehaviorComponent].
pub fn set_lookat_offset_using_input_system(
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut query: Query<
        (&mut LookatOffsetVariablesComponent,),
        With<SetLookatOffsetUsingInputBehaviorComponent>,
    >,
) {
    let mut input = Vec2::ZERO;
    for mouse_event in
        mouse_motion_events.read()
    {
        input.y +=
            mouse_event.delta.y * 0.001;
    }

    for mut lookat_offset_variables in
        query.iter_mut()
    {
        lookat_offset_variables
            .0
            .translation_wrt_parent
            .y += input.y * 0.2
    }
}

// REGIONEND
