# DEE 36-row reference-attempt correction

Status: 36_CLEAN_TEMP_FALSE_ATTEMPT_INVALID_CONFIG; CLEAN_TEMP_TRUE_PATH_UNVERIFIED; DEE_REFERENCE_NOT_READY

The 36-row exact-control attempt used an adapted XML hash that matches a reproduced preflight document containing literal placeholder paths (<INPUT_FILE>, <INPUT_PATH>, <OUTPUT_FILE>, <OUTPUT_PATH>, <TEMP_PATH>). Do not count its 0xC0000005 crash as a 36-row rejection; the source path was not validly bound.

An earlier 36-row run with clean_temp=true also crashed, but its package does not include path readback proving the adapted XML contained actual paths. Keep that attempt at CONFIG_PATH_BINDING_NOT_ESTABLISHED; it does not establish row acceptance.

A corrected 38-row run used the exact matched-control template, clean_temp=false, and path-only substitutions verified equal after path normalization. It still crashed with 0xC0000005 and produced no EC3, so 38-row acceptance was not evaluated in that run either. A separate success receipt binds output 80c3... to a 38-row input and the matched XML hash, but it lacks host image and execution-context identity and the output was not rehashed.

The earlier 36/38 crash pair is superseded. Current Wash evidence supports neither a 36-row pass nor rejection. R4 is checking whether the initial clean_temp=true attempt had valid paths; no further run should repeat a placeholder configuration.
