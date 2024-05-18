use bevy::{
    ecs::{
        bundle::Bundle,
        component::Component,
        entity::Entity,
        event::EventReader,
        query::With,
        system::{Query, Res},
    },
    input::mouse::{MouseMotion, MouseWheel},
    math::{Quat, Vec2, Vec3},
    time::Time,
    transform::components::Transform,
};

use crate::math::{CylindricalCoordinates, FromCylindrical};

#[derive(Component)]
pub struct CameraBodyTagComponent;

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

#[derive(Component)]
pub struct CameraEyesTagComponent;

#[derive(Bundle)]
pub struct CylinderCameraEyesBundle {
    pub tag: CameraEyesTagComponent,
    pub desired_transform_variables: DesiredTransformVariablesComponent,
    pub lookat_variables: LookatVariablesComponent,
    pub cylinder_coordindates_for_desired_transform_translation_variables:
        CylinderCoordinatesForDesiredTransformTranslationVariablesComponent,
    pub set_cylinder_coordinate_for_desired_transform_translation_angle_using_input_behavior:
        SetCylinderCoordinateForDesiredTransformTranslationUsingInputBehaviorComponent,
    pub set_lookat_position_to_parent_transform_translation_behavior:
        SetLookatPositionToParentTransformTranslationBehaviorComponent,
}

// REGION variables component

#[derive(Component)]
pub struct DesiredTransformVariablesComponent {
    pub desired_transform: Transform,
}

#[derive(Component)]
pub struct ObservedEntityVariablesComponent {
    pub entity: Entity,
}

#[derive(Component)]
pub struct LookatVariablesComponent {
    pub position: Vec3,
    pub up: Vec3,
}

#[derive(Component)]
pub struct CylinderCoordinatesForDesiredTransformTranslationVariablesComponent {
    pub cylinder_coordindates: CylindricalCoordinates,
}

// REGIONEND

// REGION behavior component

#[derive(Component)]
pub struct SetDesiredTransformTranslationToObservedEntityTransformTranslationBehaviorComponent;

#[derive(Component)]
pub struct SetDesiredTransformRotationToObservedEntityLocalUpBehaviorComponent;

#[derive(Component)]
pub struct SetLookatPositionToParentTransformTranslationBehaviorComponent;

#[derive(Component)]
pub struct SetCylinderCoordinateForDesiredTransformTranslationUsingInputBehaviorComponent;

// REGIONEND

// REGION apply system

pub fn apply_desired_transform_to_transform_system(
    time: Res<Time>,
    mut query: Query<(&DesiredTransformVariablesComponent, &mut Transform)>,
) {
    for (desired_transform_variables, mut transform) in query.iter_mut() {
        let next_rotation = Quat::slerp(
            transform.rotation,
            desired_transform_variables.desired_transform.rotation,
            time.delta().as_secs_f32() * 3.33,
        );

        transform.translation = desired_transform_variables.desired_transform.translation;
        transform.rotation = next_rotation;
    }
}

pub fn apply_lookat_to_transform_system(
    mut query: Query<(&LookatVariablesComponent, &mut Transform)>,
) {
    for (lookat_variables, mut transform) in query.iter_mut() {
        transform.look_at(lookat_variables.position, lookat_variables.up);
    }
}

pub fn apply_desired_transform_using_cylinder_coordinates_system(
    mut query: Query<(
        &CylinderCoordinatesForDesiredTransformTranslationVariablesComponent,
        &mut DesiredTransformVariablesComponent,
    )>,
) {
    for (cylinder_coordinates_for_desired_transform, mut desired_transform_variables) in
        query.iter_mut()
    {
        let next = Vec3::from_cylindrical(
            &cylinder_coordinates_for_desired_transform.cylinder_coordindates,
        );

        desired_transform_variables.desired_transform.translation = next;
    }
}

// REGIONEND

// REGION behavior system

pub fn set_desired_transform_translation_to_observed_entiy_transform_translation_behavior_system(
    mut query: Query<
        (
            &ObservedEntityVariablesComponent,
            &mut DesiredTransformVariablesComponent,
        ),
        With<SetDesiredTransformTranslationToObservedEntityTransformTranslationBehaviorComponent>,
    >,
    observed_query: Query<(&Transform,)>,
) {
    for (
        &ObservedEntityVariablesComponent {
            entity: observed_entity,
        },
        mut desired_transform,
    ) in query.iter_mut()
    {
        let observed_entity_transform = observed_query
            .get(observed_entity)
            .expect("Observed entity despawned!");

        desired_transform.desired_transform.translation = observed_entity_transform.0.translation;
    }
}

pub fn set_desired_transform_rotation_to_observed_entity_local_up_behavior_system(
    mut query: Query<
        (
            &ObservedEntityVariablesComponent,
            &mut DesiredTransformVariablesComponent,
        ),
        With<SetDesiredTransformRotationToObservedEntityLocalUpBehaviorComponent>,
    >,
    observed_query: Query<(&Transform,)>,
) {
    for (
        &ObservedEntityVariablesComponent {
            entity: observed_entity,
        },
        mut desired_transform,
    ) in query.iter_mut()
    {
        let observed_entity_transform = observed_query
            .get(observed_entity)
            .expect("Observed entity despawned!");

        let rotation = Quat::from_rotation_arc(Vec3::Y, *observed_entity_transform.0.local_y());

        desired_transform.desired_transform.rotation = rotation;
    }
}

pub fn set_lookat_position_to_parent_transform_translation_behavior_system(
    mut query: Query<
        (&mut LookatVariablesComponent,),
        With<SetLookatPositionToParentTransformTranslationBehaviorComponent>,
    >,
) {
    for mut lookat_variables in query.iter_mut() {
        lookat_variables.0.position = Vec3::ZERO;
    }
}

/// set desired height and lookat for player camera eyes.
pub fn set_cylinder_coordinate_for_desired_transform_translation_using_input_system(
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut query: Query<
        (&mut CylinderCoordinatesForDesiredTransformTranslationVariablesComponent,),
        With<SetCylinderCoordinateForDesiredTransformTranslationUsingInputBehaviorComponent>,
    >,
) {
    for mut desired_cylinder_coordinates_for_transform in query.iter_mut() {
        let mut input = Vec2::ZERO;
        for mouse_event in mouse_motion_events.read() {
            input.x += mouse_event.delta.x * 0.001;
            input.y += mouse_event.delta.y * 0.001;
        }

        let mut zoom_input: f32 = 0.0;
        for mouse_event in mouse_wheel_events.read() {
            zoom_input += mouse_event.y * 0.1;
        }

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

// REGIONEND
