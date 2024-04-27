# STORY As a user, I want camera coordinates to transition over time

## ACCEPTANCE

I can see the desired camera coordinate gizmo

## DESIGN

I should store the desired component state in a "transition component" and animate in the "camera_coordinate_transition_system"

// parameters for camera transition
DesiredCameraTransitionParametersComponent
easing_function:

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

# ACTION

show off the current build
Publish on Twitch under my personal account

---

## STORY physics body

implement floating ball model as described in DESIGN below

### ACCEPTANCE

I can see Sonic's up direction in a Gizmo
I can see Sonic's down direction in a Gizmo
I can see Sonic's forward direction (speed) in a Gizmo
I can see Sonic's down raycast in a Gizmo
I can see Sonic's body (floating sphere) in a Gizmo
I can see Sonic's center (floating dot) in a Gizmo

### DESIGN

Sonic is a "floating ball". the center of this ball is a "point", which is the location of his body
he uses raycasting to hover above the stage
this is like Bud from Grow Up

Sonic has a stage collider.
The stage collider is very small!

Sonic's movement is entirely kinematic, except his stage collider.

sonic has an "up" direction and a "down" direction
sonic's "up" direction is the normal of the mesh he is on, and the up is the opposite direction.
when on a mesh, sonic snaps to the mesh below his down direction (this means it must be less than 90 degrees)
when sonic leaves a mesh, his up and down directions get reset.

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

I should use an OBJ file

## ACCEPTANCE

- I can move freely in a "isosphere" model
- I can move freely around an isosphere model

---

# STORY mesh movement, prevent leaving mesh

## ACCEPTANCE

I will know this is working when:

- I stop on the edge of the mesh

## DESIGN

- cannot leave the mesh collider (will become "GroundLedgeTrip" aka "otto" machine)

## IMPLEMENTATION

- I should consider collision test to solve my "edge" problem: project point with the final position of the character's translation, in order to find the point which was left
