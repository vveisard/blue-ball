# STORY cuboid movement

## TODO fix colliding

somehow I'm getting stuck inside boxes despite checking. System ordering problem>?

## TODO fix global input direction

somehow this has a vertical (Y) component

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

# ACTION

show off the current build
Publish on Twitch under my personal account

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

Show off

---

# STORY as a user, I want to parameterize smooth_time using transition parameters

`PlayerCameraTransitionCameraStateParemters`

---

# STORY mesh movement, prevent leaving mesh

## ACCEPTANCE

I will know this is working when:

- I stop on the edge of the mesh

## DESIGN

- cannot leave the mesh collider (will become "GroundLedgeTrip" aka "otto" machine)

## IMPLEMENTATION

- I should consider collision test to solve my "edge" problem: project point with the final position of the character's translation, in order to find the point which was left
