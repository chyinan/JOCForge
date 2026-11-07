# RF F98 candidate R14 lineage boundary

Within one candidate function window, its entry RCX is copied to R14 and R14 is then passed as the candidate's first argument. No later explicit R14 definition appears in that bounded function, but nine calls intervene; continuity relies on the platform's nonvolatile-register ABI and is a static inference.

The selected outer callsite passes its own incoming RCX into this function. Thus the candidate parameter traces back to that outer RCX conditional on ABI preservation. The outer RCX is not typed or joined to the object that produced the F98 vector. Producer-owner identity, F98 entry read/type, StringType mapping, and numeric RF state remain UNBOUND. No caller-of-caller or alternate path was followed.
