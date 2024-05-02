# STORY stage loading

## ACCEPTANCE

- black screen while loading
- load into the level with the entitystage already spawned

## TODO delay spawn of character until SetupStage

Probably OnEnter SetupNextStage

## TODO configure all my system in a "play system set"

which only run while in AppState::Play

## TODO configure bevy render system sets to only work when Playing

---

# STORY render camera height and rotation gizmos

## DESIGN

this is:

- a line from the center of the cylinder to the camera
- a line from the bottom of the cylinder to the camera

---

# STORY gamepad controls

---

# STORY logo

---

# ACTION

basic show off
Publish on itch.io under my personal account
share on discord
share on 4chan/vg/sthg/
share with discord

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

# STORY As a user, I want character orientation to drive camera roll

## DESIGN

3 ranges:

- up
- lerp
- down

in the up and down ranges, the camera will snap to vertical
in the middle 45 degree range, the camera will rotate with the character

---

# ACTION

show off the current build

---

---

---

for fuark engine:

# STORY as a developer, I want phase machine

Refactor to use states.

## ACCEPTANCE

I will know this is complete when:

- as a developper, I can see the current phase

---

# STORY as a player, I want animations

## DESIGN

walking animation speed should be determined by speed

---

# STORY as a player, I want running states

## DESIGN

running uses a _horizontal direction_ and _velocity_ .

A "running" character has a desired direction (rotates towards) and desired speed.

the direction and magnitude of the stick determines this

---

# STORY as a player, I want to run along walls

---

# STORY as a player, I want slope velocity

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

# STORY optimize update_character_movement_velocity_while_on_stage_system

I'm doing an unnecessary transformation. I need to go from camera up to character up.

---

# STORY as a user, I want to parameterize smooth_time using transition parameters

`PlayerCameraTransitionCameraStateParemters`

---

# STORY

I want to include gravity while on slopes.

Gravity should increase maximum speed and acceleration (projected along the Y direction)
