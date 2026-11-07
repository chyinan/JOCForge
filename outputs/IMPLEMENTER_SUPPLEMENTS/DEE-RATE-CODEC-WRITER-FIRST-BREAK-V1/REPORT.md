# Rate-derived codec configuration to writer boundary

## Bound static path

On the inherited mode-two route, a rate code is consumed to construct a typed codec-configuration object, which is stored in a rate-derived local field. The same process function later obtains a writer-state pointer from a separate field and passes it to a codec writer routine. That routine dispatches through a per-stream vtable slot +0x28 selected from a table.

## First break

No typed pointer/owner path joins the rate-derived configuration object to the separate writer-state object. The concrete per-stream writer target is also dynamic. The evidence does not show that the rate selector changes any particular BSI bitrate field. Actual mode selection, encoder instance, and generation remain unobserved.

The upstream checker passed normally and under python -O. The rate lookup table, other processor branches, DEE runtime, media/payload/key/license access, and implementation repository were not revisited or accessed. This supplement contains no raw trace or source.
