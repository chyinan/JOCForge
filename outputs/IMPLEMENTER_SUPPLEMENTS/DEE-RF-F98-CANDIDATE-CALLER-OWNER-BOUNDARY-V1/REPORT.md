# RF F98 candidate caller owner boundary

One direct caller passes its R14 value as the candidate function's first argument. The bounded callsite contains no typed source connecting that value to the object whose F98 vector header was produced in the sealed parent.

The call edge is bound, but candidate-param1 to producer-vector owner remains UNBOUND. The candidate's F98 qword access is not established as a read of the produced vector. Entry element read/type, StringType mapping, and numeric RF state remain UNBOUND. No caller-of-caller or helper body was followed; no runtime or media/payload/key/license access occurred.
