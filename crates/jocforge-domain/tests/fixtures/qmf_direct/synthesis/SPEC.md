# QMF synthesis direct vectors v1

Status: `IMPLEMENTATION_ACCEPTANCE_VECTORS; WHOLE_RECEIVER_NOT_RUN`

Use this only for fixed mode-1 synthesis:

- phase the first 24 complex slots from delay plus current input as specified;
- update the ten-slot delay with the last ten unrotated current slots;
- apply 21-tap band0 FIR using 20 complex history slots plus 24 current slots, replace band0, and retain the last 20;
- synthesis preformat and inverse fixed 64-complex FFT use no analysis scale;
- postmodulation applies the specified rotation and factor 2;
- polyphase uses the exact `g=floor(k/4)`, `lane=k%4`, `h=36*g+lane` traversal and updates synthesis history.

The literal vectors cover stage outputs and next delay/FIR/synthesis state. They are expected assertions after fresh computation, never runtime state seeds.

Mode-0 fitting is a distinct state owner. It does not inherit mode-1 phase/FIR/band0 overwrite/synthesis state.
