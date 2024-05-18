# STORY refactor camera

## TODO documentation

## TODO lookat height

## TODO refactor common components and systems to common source files

---

# STORY refactor character ECS to match camera ECS

I switched up paradigms.

systems:

- apply
- transition
- set

components:

- variables
- behavior

## ACCEPTANCE

I will know this is working when existing paradigms are met.

---

# STORY I want a custom character controller

I need a fully custom character controller. Research needs to be done on how to achieve this.

## ACCEPTANCE

I will know this is complete when:

- I can freely move around the mesh
- I can freely move up some steps
- I can stop when an amount of speed (option) into the wall (Bonk)
- I can jump back up onto the platform with a large "step" (LedgeGetup)
- I can snap up to the top of a box (ObstacleVault)
- I have my movemnt cancelled instead of walking off the stage (alway ledgecatch)

## IMPLEMENTATION

A move_shape which returns some results is idea.

---

---

---

# STORY

combine into Sonic Adventure 3Z package

Importantly this will involve updating code to handle multiple players

---

# STORY gamepad controls

---

# STORY logo

---

# STORY as a player, I want animations

## DESIGN

walking animation speed should be determined by speed

---

# ACTION

basic show off
share on 4chan/vg/sthg/
share on project discord
share on my YouTube

---

---

---

for fuark engine:

---

# STORY as a developer, I want state machine

Refactor to use states.

## ACCEPTANCE

I will know this is complete when:

- as a developer, I can see the current phase

---

# STORY as a player, I want running states

## DESIGN

running uses a _horizontal direction_ and _velocity_ .

A "running" character has a desired direction (rotates towards) and desired speed.

the direction and magnitude of the stick determines this

---

# STORY as a player, I want slope movement velocity

## DESIGN

characters should have gravitational acceleration applied to their horizontal velocity while on a slope. Characters should have a harder time accelerating up slopes.

I think it makes sense to apply a constant "acceleration" for slopes.

---

# STORY as a player, I want to run along walls

---

# STORY I want ledge pullups

---

# STORY I want a real level with camera zones

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

# STORY as a user, I want to parameterize smooth_time using transition parameters

`PlayerCameraTransitionCameraStateParemters`

---

# STORY

I want to include gravity while on slopes.

Gravity should increase maximum speed and acceleration (projected along the Y direction)

---

# STORY I want max distance on camera cylinder origin transition

## DESIGN

Add a "cylinder transition parameter component" which are used in transition systems to change behavior of that system

## ACCEPTANCE

I will know this is working when my laggy camera origin transform is clamped to a maximum distance (Vec3)
