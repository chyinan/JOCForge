from __future__ import annotations

import json
import textwrap
from pathlib import Path


MAP_PATH = Path("docs/implementation/IMPLEMENTATION-MAP.json")
with MAP_PATH.open("r", encoding="utf-8", newline="") as handle:
    text = handle.read()
data = json.loads(text)


def entry(functional_id: str) -> dict[str, object]:
    return next(item for item in data["functional_entries"] if item["functional_id"] == functional_id)


def slice_row(functional_id: str, slice_id: str) -> dict[str, object]:
    return next(item for item in entry(functional_id).get("direct_slices", []) if item["slice_id"] == slice_id)


def find_entry_span(source: str, functional_id: str) -> tuple[int, int]:
    token = f'"functional_id": "{functional_id}"'
    position = source.index(token)
    start = source.rfind("{", 0, position)
    length = json.JSONDecoder().raw_decode(source[start:])[1]
    return start, start + length


def find_slice_span(source: str, functional_id: str, slice_id: str) -> tuple[int, int]:
    entry_start, entry_end = find_entry_span(source, functional_id)
    token = f'"slice_id": "{slice_id}"'
    position = source.index(token, entry_start, entry_end)
    start = source.rfind("{", entry_start, position)
    length = json.JSONDecoder().raw_decode(source[start:])[1]
    return start, start + length


def replace_entry_text(source: str, functional_id: str, value: dict[str, object]) -> str:
    start, end = find_entry_span(source, functional_id)
    line_start = source.rfind("\n", 0, start) + 1
    indent = source[line_start:start]
    formatted = textwrap.indent(json.dumps(value, ensure_ascii=False, indent=2), indent)
    return source[:line_start] + formatted + source[end:]


def replace_slice_text(
    source: str, functional_id: str, slice_id: str, value: dict[str, object]
) -> str:
    start, end = find_slice_span(source, functional_id, slice_id)
    line_start = source.rfind("\n", 0, start) + 1
    indent = source[line_start:start]
    formatted = textwrap.indent(json.dumps(value, ensure_ascii=False, indent=2), indent)
    return source[:line_start] + formatted + source[end:]


def find_matching(source: str, start: int, opening: str, closing: str) -> int:
    depth = 0
    quoted = False
    escaped = False
    for position in range(start, len(source)):
        char = source[position]
        if quoted:
            if escaped:
                escaped = False
            elif char == "\\":
                escaped = True
            elif char == '"':
                quoted = False
        elif char == '"':
            quoted = True
        elif char == opening:
            depth += 1
        elif char == closing:
            depth -= 1
            if depth == 0:
                return position
    raise RuntimeError("Unclosed JSON collection")


def insert_slice_text(source: str, functional_id: str, value: dict[str, object]) -> str:
    entry_start, entry_end = find_entry_span(source, functional_id)
    token = f'"slice_id": "{value["slice_id"]}"'
    try:
        source.index(token, entry_start, entry_end)
        return source
    except ValueError:
        pass
    try:
        array_key = source.index('"direct_slices":', entry_start, entry_end)
    except ValueError:
        return replace_entry_text(source, functional_id, entry(functional_id))
    array_open = source.index("[", array_key, entry_end)
    array_close = find_matching(source, array_open, "[", "]")
    property_line = source.rfind("\n", 0, array_key) + 1
    property_indent = source[property_line:array_key]
    if source[array_open + 1 : array_close].strip():
        first = source.index("{", array_open + 1, array_close)
        item_line = source.rfind("\n", 0, first) + 1
        item_indent = source[item_line:first]
        close_line = source.rfind("\n", 0, array_close) + 1
        close_indent = source[close_line:array_close]
        formatted = textwrap.indent(json.dumps(value, ensure_ascii=False, indent=2), item_indent)
        return (
            source[:close_line].rstrip()
            + ",\n"
            + formatted
            + "\n"
            + close_indent
            + source[array_close:]
        )
    formatted = textwrap.indent(json.dumps(value, ensure_ascii=False, indent=2), property_indent + "  ")
    return source[: array_open + 1] + "\n" + formatted + "\n" + property_indent + source[array_close:]


def replace_slice(functional_id: str, slice_id: str, changes: dict[str, object]) -> None:
    item = slice_row(functional_id, slice_id)
    item.update(changes)


def append_slice(functional_id: str, value: dict[str, object]) -> None:
    target = entry(functional_id)
    if any(row.get("slice_id") == value["slice_id"] for row in target.get("direct_slices", [])):
        raise RuntimeError(f"Duplicate direct slice {functional_id}/{value['slice_id']}")
    target.setdefault("direct_slices", []).append(value)


def outcomes(codes: list[str]) -> list[dict[str, str]]:
    return [
        {
            "error_code": code,
            "category": "typed producer boundary",
            "trigger": "A required source, owner, callback, shape, or numeric invariant is invalid.",
            "programme_state_after": "No partial callback/source proposal is returned; committed owner state is unchanged.",
            "caller_action": "Reject the complete staged callback and defer commit to the existing parent.",
        }
        for code in codes
    ]


METER_BRIDGE = "7ee7d1577ed3c215e652aed64065f5bcbbabfdc4ff68a46b957676e35737a9ac"
METER_FIELDS = "ea35435b13049df6357b901f75a5d8cdab46b849b392ba3867135bc679af2fc2"
WIRING = "6d1fa51b3b04ea8ca19c9d81c51b75a49d27349eaad2e018d6848e739f2cb5c1"
MATRIX = "97d554d78dce00348476cdf00fd5cb566e9c4327dc3b4f6418e344df3fa08d7a"
POINT_FIELDS = "67722a7847d0ad59ea384e7c27c4678909578401e0b167d563a8a455425fb1dc"
POINT_RESPONSE = "255b5a1b6d3648099ec31abd61c490b02987a5f320fcea7a3cf51d9f524343d8"
NORMAL = "513b0c7554d25a97c2a5e448ce0f7c109f75b01c80ec53d16a145db6208d1c23"
OWNER_BINDING = "b5a21ba892c071256aab26f1a4733b122cd0ba0dc3b27ad0182135a4b1c2cde3"
OUTER = "d676c206090cd103334a1a3704c5b97efdcf191a7f262b670ddaad32d5851d36"
P15_BASIS = "5bf3a586ec210261fff1fd916a8874dc41f98ee7102ac3e762ef660e5550f7e3"
FRAME = "d9085785d5484d236d4b24925a52954fc0bb01153c2e173e2d07d8f6e1b4a5f6"
SOURCE_TX = "1a825c5dab7ecd292e53ddd691244c7e4bef2c34d553631c3fe714dd1cf73290"
SHARED_ANALYSIS = "8726ae5e979abf587718df55e68f3f8a056ee26e09ed02f25a2f103f8ac79b23"
U04 = "ca235e2f373d95d5c628ca61c651d01805bb9cb14a7c3457d9a77b1d87941879"
U04_WEIGHT = "b16da90125002cebd14658240b517f5163e8b3b230cde18ca488cb526062c835"
U04_GAIN = "3dc243e1cb05b65dd5d703e5a0718fadae6618cdf44d6de4709430f247c208fc"
U04_FRAME_INPUTS = {
    "file": "outputs/IMPLEMENTER_SUPPLEMENTS/CODEC-COMPLETE-FRAME-ASSEMBLY-CONTRACT-V1/INPUT-ONLY-PLANS.json",
    "bytes": 4803,
    "sha256": "2a5b7a11cc2d429435a2b0c2fe88255f5eeffed295c0954239e8b2a6791d7d74",
}


replace_slice(
    "F03W",
    "p15-response-source-admission-and-columns",
    {
        "implementation_status": "LOCALLY_VERIFIED_TYPED_48_SOURCE_FIELDS_AND_P15_COLUMNS__PARENT_COMMIT_OPEN",
        "scope": "Binds all 48 same-callback global source rows, including raw geometry and the four installed semantic fields required by F03P, to the callback/source-origin identity. Prefix/bed values come from the installed source-record initializer; regular Object13 values are copied by UID/host from words8/0/11/12 and width9. It validates all-source mode, source identities, candidate/processing gates, and consumed response columns, then computes the selected P15 response and +0 post-audio widths. This is a noncommitting proposal; parent root admission and real ADM/W04 remain separate.",
        "implementation_files": [
            "crates/jocforge-domain/src/spatial/point_response.rs",
            "crates/jocforge-domain/src/spatial/selected_p15_geometry.rs",
            "crates/jocforge-domain/src/spatial/source_geometry_outer.rs",
            "crates/jocforge-domain/src/spatial/mod.rs",
            "crates/jocforge-domain/src/numeric/exact_trigonometry.rs",
            "crates/jocforge-domain/src/numeric/q15_coordinate.rs",
            "crates/jocforge-domain/src/numeric/mod.rs",
            "crates/jocforge-domain/src/joc/oamd/reader_target/exact_trigonometry.rs",
            "crates/jocforge-domain/src/frontend/normal_control_snapshot.rs",
            "crates/jocforge-domain/src/frontend/selected_adm_graph.rs",
        ],
        "test_modules": [
            "crates/jocforge-domain/tests/point_response.rs",
            "crates/jocforge-domain/tests/selected_p15_geometry.rs",
            "crates/jocforge-domain/src/spatial/point_response/tests.rs",
            "crates/jocforge-domain/src/spatial/source_geometry_outer/tests.rs",
            "crates/jocforge-domain/src/joc/oamd/reader_target/tests.rs",
        ],
        "parameters": {
            "source_field_mapping_supplement_manifest_sha256": POINT_FIELDS,
            "provider_files": [
                "outputs/IMPLEMENTER_SUPPLEMENTS/POINT-RESPONSE-SOURCE-FIELD-MAPPING-V1/BINDING.md",
                "outputs/IMPLEMENTER_SUPPLEMENTS/POINT-RESPONSE-SOURCE-FIELD-MAPPING-V1/INPUT-ONLY-VECTORS.json",
                "outputs/IMPLEMENTER_SUPPLEMENTS/POINT-RESPONSE-SOURCE-FIELD-MAPPING-V1/SOURCES.json",
                "outputs/IMPLEMENTER_SUPPLEMENTS/SPATIAL-POINT-RESPONSE-PRODUCER-CONTRACT-V1/CONTRACT.json",
                "outputs/IMPLEMENTER_SUPPLEMENTS/SPATIAL-P11-P15-SELECTED-BASIS-V1/SELECTED-P15-INPUT-RECIPE.json",
            ],
            "validation_boundary": "The four semantic source fields are joined from the approved installed prefix/bed record or the same-callback Object13 Control52 words; source+0 remains unproduced. No DBMD alias, processing flag substitution, point arithmetic approximation or commit is introduced.",
        },
        "vectors": {
            "source_field_mapping_supplement_manifest_sha256": POINT_FIELDS,
            "authoritative_vectors": [
                "outputs/IMPLEMENTER_SUPPLEMENTS/POINT-RESPONSE-SOURCE-FIELD-MAPPING-V1/INPUT-ONLY-VECTORS.json",
                "outputs/IMPLEMENTER_SUPPLEMENTS/POINT-RESPONSE-SOURCE-FIELD-MAPPING-V1/BINDING.md",
            ],
            "case_pointers": [
                "all 48 global sources preserve the prefix/bed versus UID/host object route",
                "Object13 word8/0/11/12 copy the four semantic fields; word9 is original width",
                "coordinate_parameter 0 and 1 are preserved independently",
                "same-callback source/value origin and held-source Q-1 lineage are separate",
            ],
        },
        "not_run": [
            "The typed producer has not been replayed on real ADM/audio input or accepted by W04.",
            "Full parent CallbackTransaction root swap/admission and encode entry remain open.",
        ],
    },
)

replace_slice(
    "F06OF",
    "meter-source-oar-callback-handoff",
    {
        "mapping_status": "MATERIALIZED_DIRECT_F06OF_TYPED_SPATIAL_COMPONENT_AND_TARGET_CONTROL_HANDOFF__PARENT_ADMISSION_OPEN",
        "implementation_status": "LOCALLY_VERIFIED_COMPONENT15_TARGET_CONTROL52_LINEAGE_AND_Q3_METER_ADAPTER__PARENT_ROOT_OPEN",
        "scope": "The application/CLI bridge binds an admitted selected ADM source frame to the F03W noncommitting spatial output: component rows and target Control52 rows are paired by exact target slot, C/d and callback identity; the original F05R56 S0 producer row/generation lineage is retained. The direct-LFE physical PCM row is joined by the prepared ADM graph, and the independent meter path uses Q normal callbacks plus three fresh-zero input drains. Main cached-drain audio/source hold remains a distinct Q+4 path. The bridge does not commit the spatial parent or enable main encode.",
        "implementation_files": [
            "crates/jocforge-cli/Cargo.toml",
            "crates/jocforge-cli/src/lib.rs",
            "crates/jocforge-cli/src/meter_source_bridge.rs",
            "crates/jocforge-application/src/meter_source_bridge.rs",
            "crates/jocforge-application/src/lib.rs",
            "crates/jocforge-domain/src/meter/mod.rs",
            "crates/jocforge-domain/src/meter/source_frame.rs",
            "crates/jocforge-domain/src/meter/lfe_delay_ring.rs",
            "crates/jocforge-domain/src/meter/meter_prepass.rs",
            "crates/jocforge-domain/src/joc/oamd/reader_target.rs",
            "crates/jocforge-domain/src/spatial/production_callback.rs",
            "crates/jocforge-domain/src/spatial/spatial_audio_state/component_stage.rs",
            "crates/jocforge-domain/src/spatial/source_geometry_outer.rs",
            "crates/jocforge-io/src/pcm/meter_source_cursor.rs",
            "crates/jocforge-io/src/pcm/riff_pcm24.rs",
        ],
        "test_modules": [
            "crates/jocforge-cli/tests/meter_source_bridge.rs",
            "crates/jocforge-domain/src/meter/lfe_delay_ring.rs",
            "crates/jocforge-domain/src/meter/meter_prepass.rs",
            "crates/jocforge-domain/src/spatial/production_callback/tests.rs",
            "crates/jocforge-domain/src/spatial/source_geometry_outer/tests.rs",
            "crates/jocforge-domain/src/spatial/spatial_audio_state/component_stage/tests.rs",
        ],
        "clock": {
            "domains": [
                "MeterCallbackIndex",
                "MeterUpstreamGeneration",
                "MeterSourcePosition",
                "MeterSampleCount",
                "MeterCallbackCount",
                "ObjectRenderCallbackGeneration",
                "PhysicalHostIndex",
                "CodecSourceIndex",
                "TargetSlotIndex",
                "HistoryProducerGeneration",
            ],
            "covered_clauses": [
                "A source frame and OAR input share callback index, generation, and normal/drain phase; component/control rows pair by target slot.",
                "The direct-LFE row is selected by the prepared ADM graph's physical host and is separate from regular target Control52 rows.",
                "Each target Control52 row retains its original S0 producer_row and producer_generation; delivery generation may be later.",
                "The independent meter schedule is Q normal callbacks plus three fresh-zero drains; it does not reuse the main Q+4 cached frame.",
                "Reader cursor and meter prepass proposals install only through the existing staged application/CLI boundary.",
                "The F03W spatial proposal remains pending until parent admission; this bridge does not commit the spatial owner root.",
            ],
        },
        "parameters": {
            "consumer_field_crosswalk_manifest_sha256": METER_FIELDS,
            "point_response_source_fields_manifest_sha256": POINT_FIELDS,
            "provider_files": [
                "outputs/IMPLEMENTER_SUPPLEMENTS/F06OF-MODEL-PCM-TO-OAR-TYPED-BRIDGE-V1/CONTRACT.md",
                "outputs/IMPLEMENTER_SUPPLEMENTS/METER-CONSUMER-FIELD-BINDING-CROSSWALK-V1/CROSSWALK.md",
                "outputs/IMPLEMENTER_SUPPLEMENTS/METER-CONSUMER-FIELD-BINDING-CROSSWALK-V1/REFERENCES.json",
                "outputs/IMPLEMENTER_SUPPLEMENTS/SPATIAL-CONTROL-TO-COMPONENT-PRODUCTION-WIRING-V1/STAGE-CONTRACTS.json",
                "outputs/IMPLEMENTER_SUPPLEMENTS/POINT-RESPONSE-SOURCE-FIELD-MAPPING-V1/BINDING.md",
            ],
        },
        "vectors": {
            "consumer_field_crosswalk_manifest_sha256": METER_FIELDS,
            "point_response_source_fields_manifest_sha256": POINT_FIELDS,
            "case_pointers": [
                "F05R56 lineage and target-slot row pairing",
                "Q+3 meter fresh-zero drain versus held main cached source origin",
                "same callback/transaction/C/d identity for component and target rows",
            ],
            "authoritative_vectors": [
                "outputs/IMPLEMENTER_SUPPLEMENTS/METER-CONSUMER-FIELD-BINDING-CROSSWALK-V1/REFERENCES.json",
                "outputs/IMPLEMENTER_SUPPLEMENTS/METER-CONSUMER-FIELD-BINDING-CROSSWALK-V1/CROSSWALK.md",
            ],
        },
        "not_run": [
            "Parent CallbackTransaction root commit/admission and main encode call remain unimplemented.",
            "Full resource-bound F06 measurement and W04 product acceptance remain NOT_RUN.",
        ],
    },
)

replace_slice(
    "F04A",
    "active-component-matrix-and-packet-constructor",
    {
        "mapping_status": "MATERIALIZED_DIRECT_SUBSLICE_F04A_MATRIX_REPRESENTATION__F03W_CALLBACK_COMPOSED",
        "implementation_status": "LOCALLY_VERIFIED_C_D_DENSE_VIEW_PREVIEW_AND_SINGLE_COMPONENT_EXECUTION__PARENT_COMMIT_OPEN",
        "scope": "Validates paired compact C[n,48]/destinations, scatters exact coefficient words into canonical D[15,48], gathers by the same d, and builds the non-test component packet. The pure marker mixer preview and component execution consume the same C/d and component-before overlap; execute is one-shot and advances component state exactly once, including empty/all-zero C. It exposes per-slot component rows for F06OF without committing the component owner.",
        "not_run": [
            "Independent real-audio or numeric mixer oracle; the stage tests are input-only structural/model cases.",
            "Parent root swap/admission, full main encode, actual PCM/ADM, native execution, and W04 acceptance.",
        ],
    },
)

replace_slice(
    "F12",
    "selected-q-frame-trial-and-keyless-crc",
    {
        "not_run": [
            "The parent-owned SourceTransaction receipt binds same-source F09/F10 plan identity; a production source-root caller remains open.",
            "F12S.8 q search is a separate direct slice.",
            "Durable output and publication are separate F15P proposals; main encode/root commit is not wired.",
        ],
    },
)
replace_slice(
    "F12",
    "selected-main-q-search",
    {
        "not_run": [
            "A typed same-source analysis/numeric-plan receipt is checked; the top-level source-parent caller remains unwired.",
            "Parent source commit, AU append, and durable publication are separate F15P operations.",
        ],
    },
)

f03w_field_slice = {
    "slice_id": "p15-semantic-source-field-binding",
    "mapping_status": "MATERIALIZED_DIRECT_F03W_P15_SEMANTIC_SOURCE_FIELD_SNAPSHOT",
    "implementation_status": "LOCALLY_VERIFIED_48_SOURCE_FIELD_IDENTITY_AND_CLOCK_BINDING__REAL_ADM_OPEN",
    "scope": "Builds the four P15 semantic source fields for all48 global source IDs from the installed prefix/bed source record or same-callback Object13 Control52 words8/0/11/12, carrying original audio width word9 and separating delivery generation from source origin. Joins object rows by exact UID/physical host/regular global source; source+0 remains unproduced. The producer returns a typed callback/source snapshot and performs no numeric panner work or owner commit.",
    "implementation_files": [
        "crates/jocforge-domain/src/spatial/source_geometry_outer.rs",
        "crates/jocforge-domain/src/spatial/mod.rs",
        "crates/jocforge-domain/src/frontend/normal_control_snapshot.rs",
        "crates/jocforge-domain/src/frontend/selected_adm_graph.rs",
    ],
    "test_modules": [
        "crates/jocforge-domain/src/spatial/source_geometry_outer/tests.rs",
    ],
    "clock": {
        "domains": [
            "CodecSourceIndex",
            "PhysicalHostIndex",
            "FrontendCallbackIndex",
            "CallbackLineageIdentity",
            "NormalCallbackCount",
        ],
        "covered_clauses": [
            "Prefix IDs0..10 and regular IDs11..47 are separate from physical host and local TrackUID owner ordinals.",
            "Object fields are copied from the same-callback Control52 bound by UID+physical host; held main cached source origin remains Q-1 while delivery callback advances.",
            "The direct-LFE host is excluded from regular global source IDs; source+0 remains unproduced.",
        ],
    },
    "parameters": {
        "kind": "HASH_BOUND_P15_SEMANTIC_SOURCE_FIELD_ADAPTER",
        "provider_files": [
            "outputs/IMPLEMENTER_SUPPLEMENTS/POINT-RESPONSE-SOURCE-FIELD-MAPPING-V1/BINDING.md",
            "outputs/IMPLEMENTER_SUPPLEMENTS/POINT-RESPONSE-SOURCE-FIELD-MAPPING-V1/INPUT-ONLY-VECTORS.json",
            "outputs/IMPLEMENTER_SUPPLEMENTS/POINT-RESPONSE-SOURCE-FIELD-MAPPING-V1/SOURCES.json",
            "outputs/IMPLEMENTER_SUPPLEMENTS/FRONTEND-NORMAL-CONTROL-SNAPSHOT-BINDING-V1/CONTRACT.md",
            "outputs/IMPLEMENTER_SUPPLEMENTS/OBJECT-TRACK-SAMPLER-OWNER-BINDING-V1/BINDING.json",
        ],
        "supplement_manifest_sha256": "67722a7847d0ad59ea384e7c27c4678909578401e0b167d563a8a455425fb1dc",
        "normal_snapshot_supplement_manifest_sha256": NORMAL,
        "owner_binding_supplement_manifest_sha256": OWNER_BINDING,
        "validation_boundary": "Hash-bound clean mapping only; synthetic/input-only rows exercise identity and field offsets, not real ADM or product acceptance.",
    },
    "vectors": {
        "kind": "HASH_BOUND_P15_SOURCE_FIELD_AND_UID_ROUTE_INPUT_ONLY_CASES",
        "authoritative_vectors": [
            "outputs/IMPLEMENTER_SUPPLEMENTS/POINT-RESPONSE-SOURCE-FIELD-MAPPING-V1/INPUT-ONLY-VECTORS.json",
            "outputs/IMPLEMENTER_SUPPLEMENTS/FRONTEND-NORMAL-CONTROL-SNAPSHOT-BINDING-V1/INPUT-ONLY-VECTORS.json",
            "outputs/IMPLEMENTER_SUPPLEMENTS/OBJECT-TRACK-SAMPLER-OWNER-BINDING-V1/INPUT-ONLY-VECTORS.json",
        ],
        "case_pointers": [
            "All48 source rows preserve prefix/bed vs UID/host Object13 semantic field routes.",
            "Object13 words8/0/11/12 map feature_mode/source_skip/spatial_enum/coordinate_parameter; width is word9.",
            "coordinate_parameter values0/1 are both preserved; source+0 is not read or fabricated.",
            "Cached main callback carries delivery g and source origin Q-1 as distinct fields.",
        ],
        "source_supplement_manifest_sha256": "67722a7847d0ad59ea384e7c27c4678909578401e0b167d563a8a455425fb1dc",
    },
    "error_outcomes": [
        {
            "error_code": "POINT_RESPONSE_SOURCE_FIELD_IDENTITY",
            "category": "semantic source field join",
            "trigger": "A physical host, TrackUID, global source ID, snapshot generation/origin, or field shape does not match the admitted source mapping.",
            "programme_state_after": "No typed point-response source snapshot is returned; owner state is unchanged.",
            "caller_action": "Reject the source proposal without substituting DBMD enum/flag aliases or row ordinals.",
        }
    ],
    "not_run": [
        "Real ADM/source record production and W04 spatial-quality acceptance.",
        "Parent callback commit/admission and main encode path.",
    ],
    "delivery_scope": "PUBLIC_CLEANROOM_NO_PRIVATE_KEY",
}

f03w_callback_slice = {
    "slice_id": "selected-spatial-production-callback-proposal",
    "mapping_status": "MATERIALIZED_DIRECT_F03W_ORDERED_NONCOMMITTING_CALLBACK_PROPOSAL__PARENT_ROOT_OPEN",
    "implementation_status": "LOCALLY_VERIFIED_ORDERED_SPATIAL_PROPOSAL_EMPTY_AND_CAPACITY_PATHS__PARENT_COMMIT_OPEN",
    "scope": "Runs the existing spatial callback stage sequence from typed frontend/source proposals through response, selector, capacity-gated U04, install, full target context, per-source F03S C/d, marker/history preview, canonical D, F05R56 target controls and one-shot F04A component output. Stages preserve one callback/transaction identity and return complete after-proposals without local commit or F06OF publication. CAPACITY_REACHED U04 uses the approved F03U producer; EXHAUSTED skips it.",
    "implementation_files": [
        "crates/jocforge-domain/src/spatial/production_callback.rs",
        "crates/jocforge-domain/src/spatial/production_callback/tests.rs",
        "crates/jocforge-domain/src/spatial/frontend_stages.rs",
        "crates/jocforge-domain/src/spatial/source_geometry_outer.rs",
        "crates/jocforge-domain/src/spatial/target_context.rs",
        "crates/jocforge-domain/src/spatial/point_response.rs",
        "crates/jocforge-domain/src/spatial/u04_refinement.rs",
        "crates/jocforge-domain/src/spatial/u04_projection.rs",
        "crates/jocforge-domain/src/spatial/spatial_audio_state/component_stage.rs",
        "crates/jocforge-domain/src/spatial/spatial_control.rs",
        "crates/jocforge-domain/src/spatial/current_features.rs",
        "crates/jocforge-domain/src/spatial/source_flags.rs",
        "crates/jocforge-domain/src/spatial/post_window_fanout.rs",
        "crates/jocforge-domain/src/spatial/mod.rs",
    ],
    "test_modules": [
        "crates/jocforge-domain/src/spatial/production_callback/tests.rs",
        "crates/jocforge-domain/src/spatial/source_geometry_outer/tests.rs",
        "crates/jocforge-domain/src/spatial/spatial_audio_state/component_stage/tests.rs",
        "crates/jocforge-domain/src/spatial/u04_refinement.rs",
    ],
    "clock": {
        "domains": [
            "FrontendCallbackIndex",
            "CallbackLineageIdentity",
            "CodecSourceIndex",
            "TargetSlotIndex",
            "HistoryProducerGeneration",
            "ReaderInvocationIndex",
        ],
        "covered_clauses": [
            "All current stage proposals share callback g, transaction lineage and prepared profile/provider identity.",
            "Main cached held-source view keeps delivery g distinct from normal source origin Q-1; independent meter drains remain Q+3 fresh-zero.",
            "Target row identity is carried through installation destination d, C[n,48], marker/history and component outputs.",
            "U04 is called only for CAPACITY_REACHED; EXHAUSTED and empty candidate routes skip it without changing selected IDs/order.",
            "The function returns all after-proposals pending; parent admission/root commit is not performed here.",
        ],
    },
    "parameters": {
        "kind": "WASHED_ORDERED_SPATIAL_CALLBACK_AND_COMPONENT_PRODUCTION_WIRING",
        "provider_files": [
            "outputs/IMPLEMENTER_SUPPLEMENTS/SPATIAL-CONTROL-TO-COMPONENT-PRODUCTION-WIRING-V1/STAGE-CONTRACTS.json",
            "outputs/IMPLEMENTER_SUPPLEMENTS/SPATIAL-CONTROL-TO-COMPONENT-PRODUCTION-WIRING-V1/CONTROL-HANDOFF.json",
            "outputs/IMPLEMENTER_SUPPLEMENTS/SPATIAL-ACTIVE-MATRIX-REPRESENTATION-ADAPTER-V1/REPRESENTATIONS.json",
            "outputs/IMPLEMENTER_SUPPLEMENTS/METER-CONSUMER-FIELD-BINDING-CROSSWALK-V1/CROSSWALK.md",
            "outputs/IMPLEMENTER_SUPPLEMENTS/POINT-RESPONSE-SOURCE-FIELD-MAPPING-V1/BINDING.md",
            "outputs/IMPLEMENTER_SUPPLEMENTS/SHARED-ANALYSIS-PARENT-IDENTITY-CROSSWALK-V1/CROSSWALK.md",
            "specs/03-u04-refinement-and-projection.md",
        ],
        "wiring_supplement_manifest_sha256": "6d1fa51b3b04ea8ca19c9d81c51b75a49d27349eaad2e018d6848e739f2cb5c1",
        "matrix_supplement_manifest_sha256": "97d554d78dce00348476cdf00fd5cb566e9c4327dc3b4f6418e344df3fa08d7a",
        "meter_crosswalk_supplement_manifest_sha256": "ea35435b13049df6357b901f75a5d8cdab46b849b392ba3867135bc679af2fc2",
        "source_fields_supplement_manifest_sha256": "67722a7847d0ad59ea384e7c27c4678909578401e0b167d563a8a455425fb1dc",
        "shared_analysis_supplement_manifest_sha256": "8726ae5e979abf587718df55e68f3f8a056ee26e09ed02f25a2f103f8ac79b23",
        "u04_source_weight_supplement_manifest_sha256": "b16da90125002cebd14658240b517f5163e8b3b230cde18ca488cb526062c835",
        "u04_gain_join_supplement_manifest_sha256": "3dc243e1cb05b65dd5d703e5a0718fadae6618cdf44d6de4709430f247c208fc",
        "validation_boundary": "The callback is a model proposal only. The source weight/gain producer and U04 capacity route are locally implemented, but parent root commit, durable source transaction and real ADM/W04 remain separate.",
    },
    "vectors": {
        "kind": "HASH_BOUND_SPATIAL_CALLBACK_EMPTY_ACTIVE_ZERO_CAPACITY_AND_IDENTITY_CASES",
        "authoritative_vectors": [
            "outputs/IMPLEMENTER_SUPPLEMENTS/SPATIAL-CONTROL-TO-COMPONENT-PRODUCTION-WIRING-V1/INPUT-ONLY-VECTORS.json",
            "outputs/IMPLEMENTER_SUPPLEMENTS/POINT-RESPONSE-SOURCE-FIELD-MAPPING-V1/INPUT-ONLY-VECTORS.json",
            "outputs/IMPLEMENTER_SUPPLEMENTS/U04-SOURCE-WEIGHT-CALLER-BINDING-V1/INPUT-ONLY-VECTORS.json",
            "outputs/IMPLEMENTER_SUPPLEMENTS/U04-GAIN-CALLER-JOIN-V1/INPUT-ONLY-JOIN-DISCRIMINATORS.json",
        ],
        "case_pointers": [
            "Empty selector result returns empty C/d while all pending owner proposals are staged once.",
            "All-zero active C preserves destination d and advances F04A component history once.",
            "CAPACITY_REACHED invokes U04 with original candidate order and preserves target identity; EXHAUSTED skips it.",
            "No parent commit, real audio, native or W04 acceptance is claimed.",
        ],
    },
    "error_outcomes": [
        {
            "error_code": "SpatialProductionCallbackError",
            "category": "ordered callback stage composition",
            "trigger": "A producer identity, clock, field shape, stage, selector/U04, target context, outer matrix, component or reader stage rejects the current callback.",
            "programme_state_after": "No complete callback proposal is returned; all owner after-values remain uncommitted.",
            "caller_action": "Reject and let the existing parent retire/abort policy decide; do not commit an intermediate stage.",
        }
    ],
    "not_run": [
        "Full main cached audio callback replay across Q+4 calls and the independent resource-bound meter Q+3 bridge are not covered by these synthetic callback tests.",
        "Parent CallbackTransaction admission/root swap and main encode entry remain unimplemented.",
        "Real ADM, native DSP parity, W04 and product quality acceptance are NOT_RUN.",
    ],
    "delivery_scope": "PUBLIC_CLEANROOM_NO_PRIVATE_KEY",
}

f12_source_receipt_slice = {
    "slice_id": "parent-shared-analysis-source-receipt",
    "mapping_status": "MATERIALIZED_DIRECT_F12_PARENT_SOURCE_IDENTITY_AND_SINGLE_ADVANCE_RECEIPT",
    "implementation_status": "LOCALLY_VERIFIED_SHARED_ANALYSIS_TO_NUMERIC_PLAN_RECEIPT__SOURCE_ROOT_CALLER_OPEN",
    "scope": "A crate-private SourceTransaction consumes exactly one staged F07 proposal into SharedAnalysisReceipt with a parent-owned SourceStageKey (attempt, transaction, source, analysis owner, before-generation) and parent-supplied after-generation. One NumericPlanReceipt borrows the same shared receipt and binds the fixed six-block F09/F10/F09LB plan and BAP provider. F12 BSI, published EMDF and candidate q trials compare the full same identity; second prepare or a second numeric plan rejects before replacement. No owner state is committed here.",
    "implementation_files": [
        "crates/jocforge-domain/src/transaction/mod.rs",
        "crates/jocforge-domain/src/transaction/source_transaction.rs",
        "crates/jocforge-domain/src/codec/selected_frame_trial.rs",
        "crates/jocforge-domain/src/codec/selected_main_q_search.rs",
    ],
    "test_modules": [
        "crates/jocforge-domain/src/codec/selected_frame_trial.rs",
        "crates/jocforge-domain/src/codec/selected_main_q_search.rs",
    ],
    "clock": {
        "domains": [
            "AttemptId",
            "CallbackTransactionId",
            "CodecSourceIndex",
            "AnalysisOwnerId",
            "AnalysisOwnerGeneration",
        ],
        "covered_clauses": [
            "The parent supplies committed before-generation and staged after-generation; the receipt never derives them from source index or time.",
            "One prepare_shared_analysis binds one F07 stage and advancement_count=1; a second call rejects before mutation.",
            "One NumericPlanReceipt borrows the same SharedAnalysisReceipt and can be prepared only once per source stage.",
            "F12 BSI and published EMDF identities must equal the candidate plan identity, including attempt and generation.",
        ],
    },
    "parameters": {
        "kind": "WASHED_PARENT_SHARED_ANALYSIS_AND_SOURCE_PLAN_IDENTITY",
        "provider_files": [
            "outputs/IMPLEMENTER_SUPPLEMENTS/SHARED-ANALYSIS-PARENT-IDENTITY-CROSSWALK-V1/CROSSWALK.md",
            "outputs/IMPLEMENTER_SUPPLEMENTS/SHARED-ANALYSIS-PARENT-IDENTITY-CROSSWALK-V1/SOURCES.json",
            "outputs/IMPLEMENTER_SUPPLEMENTS/CODEC-SOURCE-TRANSACTION-KEYLESS-PUBLICATION-V1/OWNER-AND-COMMIT-MAP.json",
            "outputs/IMPLEMENTER_SUPPLEMENTS/CODEC-COMPLETE-FRAME-ASSEMBLY-CONTRACT-V1/SOURCE-TO-AU-TRANSACTION.json",
        ],
        "shared_analysis_manifest_sha256": "8726ae5e979abf587718df55e68f3f8a056ee26e09ed02f25a2f103f8ac79b23",
        "source_transaction_manifest_sha256": "1a825c5dab7ecd292e53ddd691244c7e4bef2c34d553631c3fe714dd1cf73290",
        "frame_transaction_manifest_sha256": "d9085785d5484d236d4b24925a52954fc0bb01153c2e173e2d07d8f6e1b4a5f6",
        "validation_boundary": "Local typed receipt and consumer joins are verified. No native receipt, new digest hierarchy, top-level CallbackTransaction coordinator, or production source-root caller is claimed.",
    },
    "vectors": {
        "kind": "INPUT_ONLY_PARENT_IDENTITY_DOUBLE_ADVANCE_AND_SOURCE_FAILURE_CASES",
        "authoritative_vectors": [
            "outputs/IMPLEMENTER_SUPPLEMENTS/SHARED-ANALYSIS-PARENT-IDENTITY-CROSSWALK-V1/SOURCES.json",
            "outputs/IMPLEMENTER_SUPPLEMENTS/SHARED-ANALYSIS-PARENT-IDENTITY-CROSSWALK-V1/CROSSWALK.md",
            "outputs/IMPLEMENTER_SUPPLEMENTS/CODEC-SOURCE-TRANSACTION-KEYLESS-PUBLICATION-V1/INPUT-ONLY-TRANSACTIONS.json",
            "outputs/IMPLEMENTER_SUPPLEMENTS/CODEC-COMPLETE-FRAME-ASSEMBLY-CONTRACT-V1/INPUT-ONLY-PLANS.json",
        ],
        "local_fixture": U04_FRAME_INPUTS,
        "case_pointers": [
            "One shared analysis receipt is reused by F06 and F08-F11; failure after stage discards all proposals.",
            "Double F07 advance rejects; only one numeric plan can be prepared per SourceTransaction.",
            "Attempt/source/generation mismatch rejects BSI and published payload before q trial.",
            "The parent source-root and generation supplier are not called by the main encode entry.",
        ],
    },
    "error_outcomes": outcomes([
        "SourceTransactionStageError::AnalysisDoubleAdvance",
        "SourceTransactionStageError::AnalysisNotPrepared",
        "SourceTransactionStageError::NumericPlanAlreadyPrepared",
        "SelectedFramePlanError::SourceIdentityMismatch",
    ]),
    "not_run": [
        "The top-level GTC/CallbackTransaction source parent has not yet supplied committed/staged analysis owner generations to this SourceTransaction.",
        "Full F07-F12 numeric source-plan production, parent root swap, main encode call, real audio and W04 remain outside this local receipt slice.",
    ],
    "delivery_scope": "USER_AUTHORIZED_NON_PRIVATE_KEY_MATERIAL",
}

f15_output_slice = {
    "slice_id": "typed-output-source-append-and-finalize",
    "mapping_status": "MATERIALIZED_DIRECT_F15P_SOURCE_APPEND_AND_TERMINAL_FINALIZE__PARENT_ROOT_OPEN",
    "implementation_status": "LOCALLY_VERIFIED_TYPED_OUTPUT_PROPOSALS_AND_DURABLE_IO_RECEIPTS__ENCODE_ROOT_OPEN",
    "scope": "The F15 output owner creates noncommitting suppressed-source, append and finalize proposals from current admitted N. Application binds F13 metadata and F12 best frame by source/dispatch/AU. CLI stages 4096-byte published frames through atomic_output: explicit-progress writes, exact length/readback/hash/CRC, flush/fsync, expected terminal ledger and pending-writer dispositions, finalize intent, no-overwrite path precondition, rename and parent-directory fsync. Post-rename parent-sync failure returns no success receipt/DONE and preserves the visible-path uncertainty; no rollback is claimed.",
    "implementation_files": [
        "crates/jocforge-domain/src/publication/mod.rs",
        "crates/jocforge-domain/src/publication/output_owner.rs",
        "crates/jocforge-domain/src/joc/emdf/metadata_owner.rs",
        "crates/jocforge-io/src/atomic_output/mod.rs",
        "crates/jocforge-application/src/output_publication.rs",
        "crates/jocforge-application/src/lib.rs",
        "crates/jocforge-cli/src/output_publication.rs",
        "crates/jocforge-cli/src/lib.rs",
    ],
    "test_modules": [
        "crates/jocforge-domain/src/publication/tests.rs",
        "crates/jocforge-domain/src/joc/emdf/metadata_owner/tests.rs",
        "crates/jocforge-io/src/atomic_output/tests.rs",
    ],
    "clock": {
        "domains": [
            "AttemptId",
            "CodecSourceIndex",
            "CodecDispatchIndex",
            "PublishedAuIndex",
            "OutputByteOffset",
            "OutputGeneration",
        ],
        "covered_clauses": [
            "Source s uses dispatch s+1; source0/1 are suppressed and sources >=2 publish AU s-2 at offset AU*4096.",
            "F13 producers and all four current-writer bodies must be ready before queue mutation; queue order [11,1,2,14], serialization [11,14,2,1].",
            "Only complete append/readback/fsync yields a durable receipt; logical AU ledger advances after the receipt.",
            "EOF performs no new source/pop/codec flush; pending writers are recorded DROP_AT_DONE after the expected Q+2 sources and Q AUs.",
            "Finalization flushes staging, non-overwrite renames and syncs the parent; post-rename sync failure does not return a successful receipt.",
        ],
    },
    "parameters": {
        "kind": "WASHED_KEYLESS_SOURCE_APPEND_AND_FINALIZE_BOUNDARY",
        "provider_files": [
            "outputs/IMPLEMENTER_SUPPLEMENTS/CODEC-SOURCE-TRANSACTION-KEYLESS-PUBLICATION-V1/CONTRACT.md",
            "outputs/IMPLEMENTER_SUPPLEMENTS/CODEC-SOURCE-TRANSACTION-KEYLESS-PUBLICATION-V1/PUBLICATION-AND-FAILURE-BOUNDARIES.json",
            "outputs/IMPLEMENTER_SUPPLEMENTS/CODEC-SOURCE-TRANSACTION-KEYLESS-PUBLICATION-V1/OWNER-AND-COMMIT-MAP.json",
            "outputs/IMPLEMENTER_SUPPLEMENTS/CODEC-SOURCE-TRANSACTION-KEYLESS-PUBLICATION-V1/CLOCK-AND-EOF-CONTRACT.json",
            "outputs/IMPLEMENTER_SUPPLEMENTS/CODEC-COMPLETE-FRAME-ASSEMBLY-CONTRACT-V1/SOURCE-TO-AU-TRANSACTION.json",
            "outputs/IMPLEMENTER_SUPPLEMENTS/CODEC-COMPLETE-FRAME-ASSEMBLY-CONTRACT-V1/FRAME-ORDER-AND-BUDGET.json",
        ],
        "source_transaction_manifest_sha256": "1a825c5dab7ecd292e53ddd691244c7e4bef2c34d553631c3fe714dd1cf73290",
        "frame_transaction_manifest_sha256": "d9085785d5484d236d4b24925a52954fc0bb01153c2e173e2d07d8f6e1b4a5f6",
        "validation_boundary": "Local model proposal/IO protocol tests pass; actual filesystem crash durability, OS-specific no-clobber race, whole-product, native and W04 acceptance are NOT_RUN.",
    },
    "vectors": {
        "kind": "HASH_BOUND_SOURCE_AU_SUPPRESSION_APPEND_AND_TERMINAL_FAILURE_CASES",
        "authoritative_vectors": [
            "outputs/IMPLEMENTER_SUPPLEMENTS/CODEC-SOURCE-TRANSACTION-KEYLESS-PUBLICATION-V1/INPUT-ONLY-TRANSACTIONS.json",
            "outputs/IMPLEMENTER_SUPPLEMENTS/CODEC-COMPLETE-FRAME-ASSEMBLY-CONTRACT-V1/SOURCE-TO-AU-TRANSACTION.json",
            "crates/jocforge-domain/src/publication/tests.rs",
            "crates/jocforge-io/src/atomic_output/tests.rs",
        ],
        "case_pointers": [
            "Sources0/1 consume complete frame scratch and advance without durable AU append.",
            "Short writes loop until complete; zero progress rejects; readback/hash/CRC and fsync precede receipt.",
            "Finalize records DROP_AT_DONE pending writers and expected staging size; existing final path is rejected.",
            "Injected post-rename parent-sync failure returns no successful receipt/DONE and preserves path visibility uncertainty.",
        ],
    },
    "error_outcomes": outcomes([
        "OutputOwnerError::SourceOrder",
        "OutputOwnerError::FrameLengthMismatch",
        "OutputIoError::WriteNoProgress",
        "OutputIoError::ReadbackMismatch",
        "DURABLE_OUTPUT_ERROR",
    ]),
    "not_run": [
        "No top-level CallbackTransaction root swap or main encode output entry calls this proposal/IO path yet.",
        "OutputOwner attempt_id String is not yet canonically mapped to transaction::AttemptId(u64); no conversion is invented.",
        "Cross-process no-clobber race, crash/power-loss durability, real audio/ADM, native, and W04 acceptance are NOT_RUN.",
    ],
    "delivery_scope": "USER_AUTHORIZED_NON_PRIVATE_KEY_MATERIAL",
}

u04_slice = {
    "slice_id": "u04-source-weight-gain-refinement-producer",
    "mapping_status": "MATERIALIZED_DIRECT_F03U_SOURCE_WEIGHT_GAIN_AND_TARGET_REFINEMENT_PRODUCER",
    "implementation_status": "LOCALLY_VERIFIED_F03U_3_TO_5_ORDERED_SOLVE_WEIGHT_AND_PROJECTION__REAL_ADM_OPEN",
    "scope": "On the existing CAPACITY_REACHED path only, consumes original candidate-order source IDs/XYZ/full U32 attrs and raw pre-selector feature[24][N], plus selected-order initial target XYZ/attrs. It performs the F03U.2 duplicate-domain preflight, F03U.3 ordered F64 B/L/H, Cholesky and forward/back solve, emits source-major gain[N][T], derives source_weight[N] by serial F03C pow32 and projects with F03U.4/.5 epsilon/order/protected-anchor rules. It does not rederive selector membership or commit target state.",
    "implementation_files": [
        "crates/jocforge-domain/src/spatial/u04_refinement.rs",
        "crates/jocforge-domain/src/spatial/u04_projection.rs",
        "crates/jocforge-domain/src/spatial/candidate_selector.rs",
    ],
    "test_modules": ["crates/jocforge-domain/src/spatial/u04_refinement.rs"],
    "clock": {
        "domains": ["CodecSourceIndex", "TargetSlotIndex"],
        "covered_clauses": [
            "Source rows stay in original candidate order; each weight/gain row is bound to the same global source ID.",
            "Targets stay in selector order and preserve their original IDs, full attributes and initial XYZ.",
            "The producer is stateless callback-local scratch; the caller invokes it only after CAPACITY_REACHED and commits nothing here.",
        ],
    },
    "parameters": {
        "kind": "APPROVED_F03U_3_4_5_AND_F03C_POW32_PRODUCER",
        "provider_files": [
            "specs/03-u04-refinement-and-projection.md",
            "outputs/IMPLEMENTER_SUPPLEMENTS/U04-SOURCE-WEIGHT-CALLER-BINDING-V1/BINDING.md",
            "outputs/IMPLEMENTER_SUPPLEMENTS/U04-SOURCE-WEIGHT-CALLER-BINDING-V1/INPUT-ONLY-VECTORS.json",
            "outputs/IMPLEMENTER_SUPPLEMENTS/U04-GAIN-CALLER-JOIN-V1/CONTRACT.md",
            "outputs/IMPLEMENTER_SUPPLEMENTS/U04-GAIN-CALLER-JOIN-V1/INPUT-ONLY-JOIN-DISCRIMINATORS.json",
        ],
        "u04_spec_sha256": U04,
        "weight_binding_manifest_sha256": U04_WEIGHT,
        "gain_join_manifest_sha256": U04_GAIN,
        "validation_boundary": "The approved F03U formulas define a local clean model producer; no native/real-ADM numeric equivalence oracle or whole callback quality PASS is claimed.",
    },
    "vectors": {
        "kind": "INPUT_ONLY_U04_CANDIDATE_ORDER_GAIN_WEIGHT_AND_PROTECTED_TARGET_CASES",
        "authoritative_vectors": [
            "outputs/IMPLEMENTER_SUPPLEMENTS/U04-SOURCE-WEIGHT-CALLER-BINDING-V1/INPUT-ONLY-VECTORS.json",
            "outputs/IMPLEMENTER_SUPPLEMENTS/U04-GAIN-CALLER-JOIN-V1/INPUT-ONLY-JOIN-DISCRIMINATORS.json",
            "crates/jocforge-domain/tests/fixtures/f04_component/unit-vectors.json",
        ],
        "case_pointers": [
            "Candidate16/selected15 proves source_weight is gathered on all original candidate IDs, including unselected candidate47.",
            "Selector mutated scratch is not source_weight input; EXHAUSTED skips refinement even when count is15.",
            "Gain rows are source-major [N][T]; normalized membership is target-major [T][N].",
            "Zero weight/gain follows epsilon path; duplicate target domain rejects; protected bit0 target retains initial XYZ.",
            "A selected ID whose initial XYZ differs from the matching candidate source row rejects.",
        ],
        "source_supplement_manifest_sha256": U04_WEIGHT,
        "gain_supplement_manifest_sha256": U04_GAIN,
    },
    "error_outcomes": outcomes([
        "U04RefinementError::InvalidInput",
        "U04RefinementError::TargetDomainChanged",
        "U04RefinementError::NumericFailure",
        "U04ProjectionError::NumericFailure",
    ]),
    "not_run": [
        "Real source callback audio, native parity, full ADM spatial quality and W04 are NOT_RUN.",
        "Owner/target installation and parent callback commit remain caller-owned.",
    ],
    "delivery_scope": "PUBLIC_CLEANROOM_NO_PRIVATE_KEY",
}
