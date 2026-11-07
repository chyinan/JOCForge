# MP4 output-format boundary

Reference-only Wash supplement for the container/output path; it does not specify audio behavior. `IMPLEMENTATION_READY=NO` for the MP4 output selection.

The schema distinguishes `/output/ec3` elementary-stream output from `/output/mp4` container output. The candidate XSD documents `output_format` values `mp4`/`dash`, and plugin/base customization fields; this XSD copy is not installation-bound.

The static host setter candidate `FUN_1401496e0` compares `output_format` against `mp4` and `dash`, stores the accepted value at member `+0x3b`, and rejects invalid values. `Mp4::Run` statically calls `Mp4::InitPlugin`, which retrieves `output_format` from property data and passes it into a local property collection. No code-call reference binds the setter object to the Run/InitPlugin object, and no concrete muxer creation/output join is established. The first break is the same-object join between typed property handling and selected muxer creation.

`override_frame_rate` has a static string branch in the setter but was not followed beyond field handling. Other MP4 fields, plugin execution, output writer, and container effect are not closed. No audio-algorithm behavior is inferred.