# STORY render camera height and rotation gizmos

## DESIGN

this is:

- a line from the center of the cylinder to the camera
- a line from the bottom of the cylinder to the camera

---

# STORY vertical movement

## ACCEPTANCE

I will know this is complete when:

- I can jump and fall (acceleration due to gravity)

## DESIGN

- a JumpMovementParameters component: initial_speed
- a FallMovementParameters component: speed_max, speed_initial, acceleration
- CharacterOnStageComponent
  - entity, normal, and feature of the stage the character is on

---

# STORY acceleration

I would like some acceleration so it doesn't feel quite so inert

---

# STORY As a user, I want character orientation to drive camera roll

## DESIGN

3 ranges:

- up
- lerp
- down

in the up and down ranges, the camera will snap to vertical
in the middle 45 degree range, the camera will rotate with the character

---

# STORY mesh movement

## TODO

add git lfs for 3D assets

## TODO add a physics mesh

https://docs.rs/bevy_rapier3d/latest/bevy_rapier3d/geometry/struct.Collider.html#method.from_bevy_mesh
https://docs.rs/bevy_rapier3d/latest/bevy_rapier3d/geometry/struct.Collider.html#method.trimesh
https://www.models-resource.com/dreamcast/sonicadventure2/model/16287/

## ACCEPTANCE

- I can move freely move around the stage mesh

---

# ACTION

show off the current build
Publish on Twitch under my personal account

---

# ACTION

Show off

---

# STORY mesh movement, prevent leaving ledge

## ACCEPTANCE

I will know this is working when:

- I stop on the edge of the mesh

## DESIGN

- cannot leave the mesh collider (will become "GroundLedgeTrip" aka "otto" machine)

## IMPLEMENTATION

- I should consider collision test to solve my "edge" problem: project point with the final position of the character's translation, in order to find the point which was left?
  This may require "edge detection"

---

# FIX fix global input direction

## DESIGN

somehow the input has a vertical (Y) component. Should I perform a projection step? I shouldn't _have to_...

---

# STORY as a user, I want to parameterize smooth_time using transition parameters

`PlayerCameraTransitionCameraStateParemters`

---
