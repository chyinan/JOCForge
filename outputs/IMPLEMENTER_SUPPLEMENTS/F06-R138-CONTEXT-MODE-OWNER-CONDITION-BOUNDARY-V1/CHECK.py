import hashlib
import json
import sys
from pathlib import Path


PACKAGE=Path(__file__).resolve().parent
PAYLOADS={"CHECK.py","EVIDENCE.json","REPORT.md","SOURCE-MANIFESTS.json","WASH-VERIFICATION.json"}
PACKAGE_ID="F06-R138-CONTEXT-MODE-OWNER-CONDITION-BOUNDARY-V1"
INCOMING="34149ece54163b4de2b1e7ff883097db8aa946b7c455da46ebc5647484de0fcb"


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
              "status":"MODE_AND_CONDITIONAL_ZERO_RULE_STATIC_BOUND; R138_PROFILE_TO_MODE_UNBOUND",
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
    require(e["typed_mode_producer"]["component_state_member_offset"]=="+0x54"
            and e["typed_mode_producer"]["xml_profile_to_component_member"]=="NOT_BOUND","mode source edge")
    require(e["context_constructor"]["mode_zero_outcome"]=={"mode":0,"update_flag":0}
            and e["context_constructor"]["mode_one_success_outcome"]=={"mode":1,"update_flag":1},"constructor outcomes")
    require(e["conditional_owner_u16"]["mode_0_update_0"].endswith("zero")
            and e["conditional_owner_u16"]["mode_1_update_1"].endswith("zero")
            and e["conditional_owner_u16"]["r138_zero_applicability"]=="UNBOUND","conditional U16 rule")
    require(e["prior_runtime_observation"]["same_as_r138_profile"] is False
            and e["prior_runtime_observation"]["transfer_to_r138"]=="PROHIBITED","prior run boundary")
    require(e["policy"]["new_dee_run"] is False and e["policy"]["key_or_license_read"] is False
            and e["policy"]["implementation_delivery"]=="REFERENCE_ONLY; NO_PRODUCT_RULE","policy")
    sources=read_json("SOURCE-MANIFESTS.json")
    require(sources["incoming_package"]["manifest_sha256"]==INCOMING,"source binding")
    verification=read_json("WASH-VERIFICATION.json")
    require(verification["incoming_manifest_sha256"]==INCOMING
            and verification["implementation_delivery"]=="REFERENCE_ONLY; NO_PRODUCT_RULE; implementation_ready=false","verification")
    print(json.dumps({"status":"VALID_PARTIAL","package_id":PACKAGE_ID,"manifest_sha256":sha256(PACKAGE/"MANIFEST.json"),"r138_mode":"UNBOUND","conditional_zero":"STATIC"},sort_keys=True))


if __name__=="__main__":
    if sys.argv[1:]==["--build-manifest"]:
        build_manifest()
    else:
        verify()
