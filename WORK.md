# STORY As a user, I want mouse wheel to drive camera distance

---

# STORY As a user, I want camera coordinates to transition over time

## ACCEPTANCE

I can see the desired camera coordinate gizmo

## DESIGN

I should store the desired component state in a "transition component" and animate in the "transition_system"

## TODO implment transition component

## TODO implment transition system, player_camera_coordinates_transition_system

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

# STORY cuboid movement

## DESIGN

Using physics engine (bevy plugin?)

- rigidbody should be entirely kinematic
- should "validate" movement before translation is applied
- my "stage collider" does not enter any other "stage colliders"

## IMPLEMENTATION

Do a shapecast of my stage collider using movement, then translate to the final position

## ACCEPTANCE

I will know this is complete when:

- I can use input to move along a sphere collider using Rapier
- I cannot enter boxes

---

# STORY mesh movement

## TODO add a physics mesh

Might be fun to import Green Hill zone from Sonic Adventure 2?

# STORY otto movement

I want to validate I do not leave the mesh collider, then stop on the edge.

## DESIGN

- cannot leave the mesh collider (will become "GroundLedgeTrip" aka "otto" machine)
- how do I stop _exactly_ on the edge?
- clearly if I became airborn I should stop, but stopping _on the edge_ is a bit more difficult...

---

# STORY I want camera zones

## DESIGN

Thinking about camera zones. Will probably take the form of a "blend" component and a system which sets the desired camera coordinate to the blended value

```
struct PlayerCameraBlendComponent {
  desired_coordinates: PlayerCameraCoordinatesComponent
}
```

camera zones have a physics trigger which adds the blend component when the character enters that camera zone.

## ACCEPTANCE

I will know this is complete when:

- I can see camera zones in the Bevy Physics render
- When in camera zones, my camera is blended between my current and the desired using a "speed" value

## TODO add Bevy to app
