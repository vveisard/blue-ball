# STORY replace camera lookat with eyes_forward

## RESOURCES

https://docs.rs/bevy/0.13.0/bevy/transform/components/struct.Transform.html#method.looking_to

## DESIGN

eyes_forward and eyes_up will be used to determine the camera rotation.

camera should look _down_ as the height moves up

## IMPLEMENTATION

I can adjust the y value of the direction using vertical mouse input, and adjust the x/z using horizontal mouse input by rotating the desired direction about the Y axis (at same speed of the angle)

---

# ACTION

show-off

---

# STORY generic camera transform

I've decided that I should create a generic camera interpolation system.

## DESIGN

the camera cylinder behavior should apply a desired transform using the cylinder coordinates.

## TODO

remove camera cylinder "desired state"

---

# STORY I want a custom character controller

I need a fully custom character controller. I should research my

## IMPLEMENTATION

A module full of code to move around, like MoveShape, is ideal.

## DESIGN

Output of MoveShape is a data structure with "segments and events". Ie, the segments that were traveled, and when events happened during that travel.

Fixed bodies should not be able to be moved into.
Dynamic bodies should be ignored, and be able to be moved into to.

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
