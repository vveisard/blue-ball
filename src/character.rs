use bevy::{
    ecs::{bundle::Bundle, component::Component},
    math::{Quat, Vec3},
    render::view::InheritedVisibility,
    transform::components::{GlobalTransform, Transform},
};

#[derive(Component)]
pub struct CharacterTagComponent;

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
    /// ie, input in camera space transformed to global space
    pub global_input: Vec3,
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
}
