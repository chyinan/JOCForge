# Helper offset and FFOA normalization

The static helper contract uses binary64 seconds. Its master start_time defaults to 0; relative FFOA defaults to -1.0 as a missing sentinel. Three signed 32-bit components are converted as c0*3600 + c1*60 + c2/100000 seconds. Negative relative FFOA is normalized to zero. The helper reports offset=start_time and absolute FFOA=start_time+max(relative_ffoa,0). The host initializes output slots to zero and leaves a slot at zero if its corresponding helper output line is absent.

For the full-hash-identified sample-6 ADM, the prior native receipt reports offset=ffoa=3610 seconds, and the AXML programme start is also 3610 seconds. If those native values are applied to the approved file-position range equation, the start sample is zero. This supports the exact sample-6 full-file input.

The static evidence does not show who writes master start_time or prove that AXML programme.start is that writer. It also does not bind the helper image used by the prior host run. Do not generalize the sample-specific match into an auto resolver for other ADM files. Overflow and NaN/Infinity rules remain unobserved.
