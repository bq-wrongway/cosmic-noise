Result:

Set of output segments
Partial order on that set
Each input segment maps to a sequence of instances. An instance is an output segment and a direction. Continuity: internal endpoints join, extremal endpoints match input. Frechet distance of path is within epsilon of input. 

Note an output segment may appear in multiple instances.

Forall (x0, y) \in seg0 and (x1, y) in \seg1, seg0 is ordered wrt seg1 in the po, and either both are endpoints with x0 = x1, or x0 and x1 are strictly ordered consistent with po.

Strengthen to robust ordering. Need to get the details right, but same logic as existing writing. Horizontal line segments are special-cased.

Active list respects po.
