# Notes on robust path operations

## Point-line orientation

Capsule case:

p1 = line.nearest(p)
|p1 - p| > epsilon, orientation is p.x ? p1.x

Scanline variant (TODO what to call this?):

Same as capsule case if oriented
Else if p.y = l.p0.y, p.x ? l.p0.x
     if p.y = l.p1.y, p.x ? l.p1.x

Note that this is the same capsule case, eps goes to 0 at endpoints.

## Line-line orientation

l0 is top-oriented with l1 iff their top y are equal, *or* the samples at l0.p0.y.max(l0.p0.y) are both capsule-oriented.

Bottom-oriented similarly.

Active list invariant:

For each successive pair of lines, they are top-oriented >= and bottom-oriented >=.

