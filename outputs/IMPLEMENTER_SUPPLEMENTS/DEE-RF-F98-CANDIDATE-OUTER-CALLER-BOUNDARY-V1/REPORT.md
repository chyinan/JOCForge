# RF F98 candidate outer-caller boundary

The bounded outer callsite passes the caller's incoming RCX unchanged as the candidate function's entry RCX. A separate sealed inner callsite passes R14 as the candidate's first argument. The caller window does not establish that entry RCX becomes R14.

No typed owner edge joins the incoming object to the producer object whose F98 vector was initialized. The candidate F98 access remains unassigned to that producer vector. Entry element type, RF StringType mapping, and numeric state remain UNBOUND. No caller-of-caller or alternate callsite was followed.
