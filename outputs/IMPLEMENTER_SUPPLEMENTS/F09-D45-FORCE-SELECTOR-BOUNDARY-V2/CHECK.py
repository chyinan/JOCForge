import hashlib
import json
import sys
from pathlib import Path


PACKAGE=Path(__file__).resolve().parent
PAYLOADS={"CHECK.py","EVIDENCE.json","INPUT-ONLY-VECTORS.json","REPORT.md","SOURCE-MANIFESTS.json","WASH-VERIFICATION.json"}
PACKAGE_ID="F09-D45-FORCE-SELECTOR-BOUNDARY-V2"
INCOMING="331d481f03dc8d49cfebff135e521604ebf4b4012d4491f7be6e4ab93b23031c"
PARENT_WASH="e88fb02651d65da5a81515348d4d96dfc74dac3490ca2a20b91e28d1e9817114"


def require(condition,message):
    if not condition:
        raise RuntimeError(message)


def sha256(path):
    digest=hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda:stream.read(1024*1024),b""):
            digest.update(chunk)
    return digest.hexdigest()


def read_json(name):
    return json.loads((PACKAGE/name).read_text(encoding="utf-8"))


def build_manifest():
    actual={path.name for path in PACKAGE.iterdir() if path.is_file()}-{"MANIFEST.json"}
    require(actual==PAYLOADS,f"payload inventory mismatch: {sorted(actual^PAYLOADS)}")
    files=[{"file":name,"bytes":(PACKAGE/name).stat().st_size,"sha256":sha256(PACKAGE/name)} for name in sorted(PAYLOADS)]
    manifest={"schema":"jocforge-washed-implementer-supplement.v1","package_id":PACKAGE_ID,
              "status":"COLD_SOURCE_AND_FORCE_STAGING_STATIC_BOUND; WIRE_FIELD_AND_RUNTIME_OWNER_UNBOUND",
              "implementation_ready":False,"source_reverse_manifest_sha256":INCOMING,"files":files}
    (PACKAGE/"MANIFEST.json").write_text(json.dumps(manifest,indent=2,sort_keys=True)+"\n",encoding="utf-8")


def verify():
    expected=PAYLOADS|{"MANIFEST.json"}
    actual={path.name for path in PACKAGE.iterdir() if path.is_file()}
    require(actual==expected,f"file inventory mismatch: {sorted(actual^expected)}")
    manifest=read_json("MANIFEST.json")
    require(manifest["schema"]=="jocforge-washed-implementer-supplement.v1" and manifest["package_id"]==PACKAGE_ID
            and manifest["implementation_ready"] is False and manifest["source_reverse_manifest_sha256"]==INCOMING,"manifest identity")
    rows={row["file"]:row for row in manifest["files"]}
    require(len(rows)==len(manifest["files"]) and set(rows)==PAYLOADS,"manifest inventory")
    for name,row in rows.items():
        path=PACKAGE/name
        require(path.stat().st_size==row["bytes"] and sha256(path)==row["sha256"],f"payload hash {name}")
    e=read_json("EVIDENCE.json")
    require(e["package_id"]==PACKAGE_ID and e["implementation_ready"] is False,"evidence identity")
    require(e["history_copy"]["shape"]=="6 blocks x 5 lanes; block-major/lane-minor","history shape")
    s=e["force_selector"]
    require(s["six_slot_thresholds"]==[2,4] and s["wire_field_name"]=="UNKNOWN" and s["wire_bit_range"]=="UNKNOWN","selector boundary")
    vectors=read_json("INPUT-ONLY-VECTORS.json")["vectors"]
    expected_vectors={"D45_ZERO_HISTORY":0,"D45_NONZERO_RUN_BELOW_T1":3,"D45_NONZERO_RUN_AT_T1":2,"D45_NONZERO_RUN_AT_T2":1,"D45_CONTROL_PROMOTES_CODE_2":3}
    actual_vectors={v["id"]:v["expected"]["staged_code"] for v in vectors if "staged_code" in v["expected"]}
    require(actual_vectors==expected_vectors,"input-only selector vectors")
    require(e["runtime"]["attempt10_11_owner_history_breakpoints"]=="NOT_HIT"
            and e["runtime"]["attempt12_plugin_loaded"] is True
            and e["runtime"]["attempt12_failure"]=="MediaInfo unavailable before PCM-frame delivery"
            and e["runtime"]["runtime_force_values"]=="NOT_CAPTURED","runtime limit")
    require(e["policy"]["pcm_or_payload_read"] is False and e["policy"]["implementation_delivery"]=="REFERENCE_ONLY; NO_PRODUCT_RULE","policy")
    sources=read_json("SOURCE-MANIFESTS.json")
    require(sources["incoming_package"]["manifest_sha256"]==INCOMING and sources["incoming_package"]["manifest_seal_matches"] is True
            and sources["supersedes_wash_package"]["manifest_sha256"]==PARENT_WASH,"source binding")
    verification=read_json("WASH-VERIFICATION.json")
    require(verification["incoming_manifest_sha256"]==INCOMING and verification["implementation_delivery"]=="REFERENCE_ONLY; STATIC_DEE_RULES_ONLY; implementation_ready=false","verification")
    print(json.dumps({"status":"VALID_PARTIAL","package_id":PACKAGE_ID,"manifest_sha256":sha256(PACKAGE/"MANIFEST.json"),"wire_field":"UNKNOWN"},sort_keys=True))


if __name__=="__main__":
    if sys.argv[1:]==["--build-manifest"]:
        build_manifest()
    else:
        verify()
