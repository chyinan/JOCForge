# RF F98 candidate consumer boundary

One bounded candidate function reads a qword from its own argument at the F98 displacement and uses that value in pointer/offset arithmetic before a direct helper call. The inspected window does not load an element through the derived address before that helper boundary.

The candidate argument is not typed or joined to the object whose F98 vector header was produced in the sealed parent. Therefore this is not a proven consumer of that vector. Producer-owner identity, entry read/type, StringType mapping, and numeric RF state remain UNBOUND. The helper body was not followed; no global search or runtime capture is claimed.

Ghidra 12.2 read-only evidence only. No DEE/runtime, XML parse, decode, PCM, payload, key, license, or implementation access occurred.
