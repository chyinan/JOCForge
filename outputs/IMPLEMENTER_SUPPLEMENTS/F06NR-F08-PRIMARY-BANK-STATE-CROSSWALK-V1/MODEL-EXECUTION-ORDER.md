# Model bank and state order

## Two owners, one bank choice in the secondary path

The approved F07-F12 analysis owner has separate F08 `primary_transform` and `companion_transform` histories. Each family has five FBW lanes and one LFE history. They are mutable state for the analysis/root transform path.

The F06WH `main_none_drc_state.secondary` owner instead has five FBW `forward_history` lanes, five independent `inverse_carry` lanes, and `frame_index`. Its `forward_history` belongs to an independent F06NR reconstruction instance. F06NR runs the F08 PRIMARY long/short algorithm against that private history. It does not read or mutate either analysis-root history, even though its algorithm family is called PRIMARY.

The frozen F08 model makes the algorithm-bank distinction explicit: `alternate=False` produces the PRIMARY spectrum/history; `alternate=True` produces the companion spectrum/history. The F06NR contract calls the PRIMARY transform, so the source-frame pipeline uses only the first branch for the F06NR secondary computation. The root F08 computation retains its separate primary and companion histories.

## One F06NR source frame

For source frame `s`, F06NR receives the same-generation, once-delayed F07 six-row input and 30 F07 `short_consumed` flags. It iterates internal lanes `[L,C,R,Ls,Rs]`, mapped to external rows `[0,2,1,4,5]`, then blocks 0 through 5. Each flag selects long or short for the F06NR-owned F08 PRIMARY transform. Its 256 unquantized spectrum is kept for bins 0 through 216 and has bins 217 through 255 replaced by positive zero. F06NI consumes that spectrum with the same long/short flag; F06NF supplies its FFT; F06NR overlap then updates only that lane's inverse carry.

The inverse does not consume the root analysis primary or companion spectra. It consumes the F06NR-local PRIMARY spectrum produced from the delayed input with the F06NR-local forward history. The block-0 overlap output is discarded after both local histories advance. Blocks 1 through 5 populate carrier blocks 0 through 4. F06WH commits the next secondary state only after the whole source succeeds; then `frame_index` becomes `s+1`.

## Reset and status

Cold/reset F06WH secondary state is positive-zero forward history and inverse carry with `frame_index=0`. The F07-F12 analysis root separately resets its primary and companion histories. These resets clear different owner leaves; no memory alias is implied.

The source 0/1/2 input-only recipes and hashes are cross-referenced in `SOURCE-VECTOR-CROSSWALK.json`. Numeric F32 after-state values remain `NOT_RUN` here. The frozen-model bank selection is resolved. Native DLL caller identity and runtime behavior are outside this model-only addendum; no native claim is made.
