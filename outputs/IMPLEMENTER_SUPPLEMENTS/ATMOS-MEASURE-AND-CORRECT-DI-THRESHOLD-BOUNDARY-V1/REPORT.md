# Dialogue fields in measure-and-correct branch

## Wash disposition

The upstream Ghidra receipt traces the sibling `measure_and_correct` branch and separates it from `measure_only`. It does not establish that the DI or speech-threshold values control PCM gain, DialNorm, or both.

## Static branch facts

- `FUN_180210fe0` selects `measure_and_correct`, writes mode output 0, and calls the same metering parameter handler. DI and threshold enter distinct configuration slots.
- The single-pass correction workflow has a `pcm_scale` mode with a scale output and a separate `md_update` option path.
- The scoped exports do not connect the `measure_only` threshold slot `lVar16+0xd48` to the first speech-score comparison, the correction workflow's `param_2[0x13]` / PCM scale, or the `md_update` writer.

## Boundary

The relationship between speech threshold, PCM gain correction, DialNorm, and any actual meter comparison remains `UNBOUND`. The matching offset alone is not object identity. Do not merge this branch with `measure_only` behavior or invent a threshold formula. No DEE run, DSP, PCM, payload, key, or license access occurred.
