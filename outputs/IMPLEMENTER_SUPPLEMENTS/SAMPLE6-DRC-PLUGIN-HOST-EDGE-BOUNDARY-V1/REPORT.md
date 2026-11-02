# Human Cat plugin host/API output boundary

The Atmos plugin's F32 row stays plugin-local through its callback. The callback receives a plugin processing-state pointer, the plugin instance, and a per-frame output record at `instance+0xb20`; it returns an integer status. On success, the plugin wrapper passes a plugin-owned writer and buffer at `instance+0xb90` to its next serializer, with byte length `2 * U16(instance+0xb9a)`.

The traced plugin call path does not pass the plugin F32 row pointer to the E-AC-3 core encoder. The core has a separate inline six-by-five F32 row. No host function body or typed transfer from the plugin output/state to that core row was resolved in the inspected images. This bounds the plugin path; it does not prove that no such transfer exists elsewhere in the host.

The next edge is the host-side caller/consumer for the plugin callback or output record. If it does not transfer typed F32 row data into the core context, the core row needs an independent core-owned producer. No runtime callback values or media were captured.
