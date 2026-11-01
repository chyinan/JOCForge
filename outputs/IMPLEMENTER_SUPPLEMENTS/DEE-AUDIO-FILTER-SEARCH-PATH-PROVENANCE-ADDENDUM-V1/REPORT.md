# DEE audio-filter search-path provenance

## Scope

This addendum binds the static host-side producer and consumer path for plugin search-path input. It does not recover the effective ordered module list for Run145 or Run148, so neither run's absolute DDP Atmos manager/Runner ordinal is closed.

## Static path

For the hash-bound host image, `FUN_14002A0B0` reads `DEE_PLUGIN_PATH` and falls back to configuration key `dee_loader.list` when the environment value is empty. `FUN_140167960` deduplicates and appends the selected path string to the PluginManager singleton's `+0x8` member. The directory-list builder splits that string on semicolons, walks those path components in sequence, and appends candidate files returned by `FindFirstFileW`/`FindNextFileW`; the traced path has no explicit sort step. `LoadAll` then walks the resulting 0x20-byte path records in sequence and calls `LoadOne`.

Combined with the previously washed callback-order contract, the selected DDP Atmos global ordinal is conditionally `sum(preceding LoadOne modules' audio_filter getter counts) + local factory index`. This is an ordering rule, not a value for a specific run.

## Run boundary

Run145 records only the DDP Atmos image path. Run148 binds that image and process entry but lacks the complete ordered audio-filter set. Neither job XML nor the cited preflight supplies the effective `DEE_PLUGIN_PATH`/fallback value or complete `LoadOne` sequence and getter counts. The actual directory-enumeration result is not recoverable from the installation's present-day files. Therefore both absolute ordinals stay `UNBOUND`.

No runtime metadata capture, host launch, encode, media read, PCM/payload/bitstream read, or key/license access occurred in this package.
