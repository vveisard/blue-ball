# STORY player camera cylinder origin rotation transition

## DESIGN

We want to transition the rotation of the camera body towards the characters's rotation
ie, slowly transition the camera cylinder "up" from the current up to the character's up
this will use the transition system (desired state, etc)

---

# STORY I want max distance on camera cylinder origin transition

## DESIGN

Add a "cylinder transition parameter component" which are used in transition systems to change behavior of that system

## ACCEPTANCE

I will know this is working when my laggy camera origin transform is clamped to a maximum distance (Vec3)

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

# ACTION

show off the current build

---

# STORY I want a real level with camera zones

---

---

for fuark engine:

# STORY as a developer, I want state machine

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

# STORY as a player, I want slope movement velocity

## DESIGN

characters should naturally begin to accelerate down slopes. Characters should have a harder time accelerating up slopes.

I think it makes sense to apply a constant "acceleration" for slopes.

---

# STORY as a developer, I want to render only when playing

## TODO configure bevy render system sets to only work when Playing

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
