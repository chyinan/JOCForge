# Native clip peak/hold producer at the sample boundary

This internal supplement closes the native clip producer and compensation algorithm for the explicit 6-row, 1536-sample owned input interface. It does not claim production-cold parent integration or equivalence with the frozen model's target-gain clip source.

## Owner and state

The clip owner is initialized cold per independent case: previous-tail peak +0, hold counter0, envelope level +0, gain exponent2 for the selected fixture, normalization `R32(gain_exponent/48)`, previous attenuation1/16, and a separate 256-sample delay backed by a 512-sample-per-row ring. The selected wrapper receives six F32 rows of1536 samples each. LFE row3 is excluded from peak decisions but still passes through delay and compensation. One wrapper call is one state tick; hold counts calls, not samples or milliseconds.

## Peak and state update

For each call, compute absolute sample peaks over non-LFE channels. The decision peak is the max of old previous-tail peak, current prefix samples0..1279 and current tail1280..1535. The next previous-tail state is only the max of current samples1280..1535. Thus the current tail participates now and once again in the next call; prefix samples do not persist.

For exponent extraction only, clamp the nonnegative peak to[-1,1], reinterpret as F32 bits, form `u=(2*w) mod 2^32`, then `e=-127` if u==0 else `(u>>24)-126`; `n=clamp(-e,0,60)`. If n>=48 or peak==0, transformed=1. Otherwise use the original peak: `t=R32(peak*2^n)`; `z=R32(R32(A-R32(t*B))+R32(R32(t*t)*C))`; `transformed=R32(R32(z*4)+R32(R32(n)*D)`. The constants A/B/C/D are provided in FUNCTIONAL-PARAMETERS.json; this polynomial is not replaced by ideal log2.

Compute `difference=R32(transformed-normalization)`, multiply by6 in binary64, clamp to[-1,1] in binary64 and R32; `candidate=R32(clamped-1/128)`. Let `release=R32(old_level+release_step)`:

- if release>candidate, set level=candidate and hold=5;
- else if old hold is nonzero, keep old level and decrement hold;
- else set level=release and hold remains0.

The comparison is strict. Equality does not attack. No level clamp is applied.

For lookup, `x=min(next_level,0)`. Start i=0 and increment while `i<127 && x<threshold[i]`. Emit U32 clip index `128+i`, so this producer reaches128..255. All128 threshold words are explicit. The tempting formula `R32(-log2(16*attenuation)/8)` differs at36 entries and is not the approved threshold generator. All128 attenuation words are supplied; they independently match `R32((32+i%32)*2^floor(i/32)/512)`.

## Delay, attenuation and writer field

After computing clip, use `attenuation[clip-128]`. The delayed frame is previous tail256 + current prefix1280. Store current tail256 for the next call. Let `increment=R32(R32(attenuation-previous_attenuation)/256)` and `a=previous_attenuation`. For samples0..255 compute `factor=R32((1/16)/a)`, write `R32(R32(factor*sample)*2^gain_exponent)` for each row, then `a=R32(a+increment)`. For samples256..1535 use fixed `factor=R32((1/16)/attenuation)` with the same two rounded multiplies. Store previous_attenuation=attenuation. The clip U32 is written into the same context consumed by the ID14 writer; it occupies the writer's 8-bit clip field. Sequence is a separate U32.

## Scope and model difference

Native clip is generated from audio sample peaks and persistent peak/hold/release history. The frozen model's clip source is descriptor target-gain lookup and has no peak/hold state. The synthetic counterexamples show alternative inputs can differ; they are not a natural same-capture comparison. Keep these producer origins separate until a selected natural-generation receipt binds them. Do not silently replace the approved model producer or relabel target-gain as native peak history.

The 393 cases/434 generations use owned synthetic F32 arrays and explicit state fixtures; no real PCM is read, stored or hashed. They validate this sample-boundary state machine and same-context writer handoff, not QMF/fit/upstream audio or full production parent. Native thresholds and attenuation are ordinary non-secret functional values. No private key or authentication output is included. Whole F13C remains NOT_READY.
