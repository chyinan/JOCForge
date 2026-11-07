# `time_base` typed enum setter boundary

The selected DDP plugin's `time_base` option is a StringType descriptor at `Priv+0x550`. Its owner constructs a parameter container that indexes this descriptor by its `time_base` name. The plugin option-tree callback performs the key lookup, then invokes the matched descriptor's StringType setter. The setter checks the descriptor's allowed-value list and stores a matching string. The allowed values are `file_position` and `embedded_timecode`; an unmatched value follows the StringType constraint-failure path.

This closes only the plugin-side path from a key/value already in the DDP option tree to the descriptor setter. It does not close creation of that tree from original XML/job configuration. It also does not recover a runtime value from E-203: that comparator observed a nonzero result, which means only “not equal to embedded_timecode” absent same-run enum-acceptance evidence.

No runtime attempt or media was used for this static addendum. No source code, decompilation, PCM, encoded payload, key or license is included.
