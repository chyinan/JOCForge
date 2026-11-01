import hashlib
import json
import sys
from pathlib import Path


PACKAGE=Path(__file__).resolve().parent
PAYLOADS={"CHECK.py","EVIDENCE.json","REPORT.md","SOURCE-MANIFESTS.json","WASH-VERIFICATION.json"}
PACKAGE_ID="F13C-ATMOS-CALLBACK-REGISTRATION-EDGE-BOUNDARY-V1"
INCOMING="b62e4aece92a5ec21459b529c5655551ca6eb318a09b2a987b965e58f00e5e8c"
PARENT_V8="c9e10f0c97ecc00990dbc5986d3bc1fb3c310a8fc6bbd0cbba768b5dd044007a"


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
              "status":"CALLBACK_REGISTRATION_EDGE_BOUND; BRANCH_CONTINUATION_AND_WRITER_UNBOUND",
              "implementation_ready":False,"source_reverse_manifest_sha256":INCOMING,"files":files}
    (PACKAGE/"MANIFEST.json").write_text(json.dumps(manifest,indent=2,sort_keys=True)+"\n",encoding="utf-8")


def verify():
    expected=PAYLOADS|{"MANIFEST.json"}
    actual={path.name for path in PACKAGE.iterdir() if path.is_file()}
    require(actual==expected,f"file inventory mismatch: {sorted(actual^expected)}")
    manifest=read_json("MANIFEST.json")
    require(manifest["schema"]=="jocforge-washed-implementer-supplement.v1" and manifest["package_id"]==PACKAGE_ID
            and manifest["implementation_ready"] is False and manifest["source_reverse_manifest_sha256"]==INCOMING,"manifest")
    rows={row["file"]:row for row in manifest["files"]}
    require(len(rows)==len(manifest["files"]) and set(rows)==PAYLOADS,"manifest payloads")
    for name,row in rows.items():
        path=PACKAGE/name
        require(path.stat().st_size==row["bytes"] and sha256(path)==row["sha256"],f"payload {name}")
    e=read_json("EVIDENCE.json")
    require(e["package_id"]==PACKAGE_ID and e["implementation_ready"] is False,"evidence identity")
    require(e["wrapper_path"][0]["callsite"]=="0x1403fac12"
            and e["wrapper_path"][1]["child_offset"]=="+0x30"
            and "AtmosMasterFileStorage::Factory" in e["wrapper_path"][1]["target"], "wrapper/child path")
    require(e["registration_helper"]["branch_site"]=="0x1403a6984"
            and e["registration_helper"]["continuation_inspected"] is False,"branch scope")
    for key in ["selected_ddp_joc_5_1_stage","ddp_joc_ec3_frame_writer","f13c_writer"]:
        require(e["writer_reachability"][key]=="UNBOUND",f"writer edge {key}")
    require(e["policy"]["encoder_launched"] is False and e["policy"]["pcm_read"] is False
            and e["policy"]["encoded_payload_read"] is False,"execution scope")
    sources=read_json("SOURCE-MANIFESTS.json")
    require(sources["incoming_package"]["manifest_sha256"]==INCOMING
            and sources["parent_source_package"]["manifest_sha256"]==PARENT_V8,"source chain")
    verification=read_json("WASH-VERIFICATION.json")
    require(verification["incoming_manifest_sha256"]==INCOMING
            and verification["implementation_delivery"]=="REFERENCE_ONLY; NO_PRODUCT_RULE; implementation_ready=false","verification")
    print(json.dumps({"status":"VALID_PARTIAL","package_id":PACKAGE_ID,"manifest_sha256":sha256(PACKAGE/"MANIFEST.json"),"writer":"UNBOUND"},sort_keys=True))


if __name__=="__main__":
    if sys.argv[1:]==["--build-manifest"]:
        build_manifest()
    else:
        verify()
