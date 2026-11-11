# Flat7x metadata reference fixture

`official_demo_oamd_position_reference.json` records the sanitized, full-track
OAMD position observation from the Wash-reviewed 1,280-kbps reference sample.
Its manifest and package seal are recorded in the fixture and in
`docs/memory/shared/evidence-index.md`.

The fixture keeps three observed domains separate: absolute position including
room-coordinate Z, the `b_enable_elevation` zone constraint, and object-size
index/resolved height. The 1,280-kbps sample is a metadata reference; the
1,664-kbps target remains `NOT_EVALUATED`, and `production_allowed` remains
false. The fixture supplies no coordinates, metadata recipe, or encoding rule.
