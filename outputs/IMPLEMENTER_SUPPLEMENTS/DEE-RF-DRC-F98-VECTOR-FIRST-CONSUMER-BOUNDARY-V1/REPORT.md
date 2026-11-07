# RF DRC vector first-consumer boundary

The bounded same-caller continuation after a vector copy into the owner field at +0xf98 contains 22 remaining instructions and no later exact reference to that field. The next direct call receives a different explicit destination field, +0xfb0, with separate stack arguments. The evidence does not join that vector to the earlier +0xf98 vector by offset proximity or call similarity.

No consumer or serialization point for +0xf98 was found within this caller. The owner/consumer join, RF DRC encoding setup, and numeric RF effect remain UNBOUND. This is a one-caller boundary only; no global absence claim or runtime execution is made.
