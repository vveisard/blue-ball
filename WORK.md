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

# STORY mesh movement

## TODO add a physics mesh

Might be fun to import Green Hill zone from Sonic Adventure 2?

# STORY otto movement

I want to validate I do not leave the mesh collider, then stop on the edge.

## DESIGN

- cannot leave the mesh collider (will become "GroundLedgeTrip" aka "otto" machine)
- how do I stop _exactly_ on the edge?
- clearly if I became airborn I should stop, but stopping _on the edge_ is a bit more difficult...
