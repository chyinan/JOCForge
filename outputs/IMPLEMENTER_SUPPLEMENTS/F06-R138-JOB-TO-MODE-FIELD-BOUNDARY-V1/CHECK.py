import hashlib
import json
import sys
from pathlib import Path


PACKAGE=Path(__file__).resolve().parent
PAYLOADS={"CHECK.py","EVIDENCE.json","REPORT.md","SOURCE-MANIFESTS.json","WASH-VERIFICATION.json"}
PACKAGE_ID="F06-R138-JOB-TO-MODE-FIELD-BOUNDARY-V1"
INCOMING="47c0931c8843cbcc2ae664b3d57168597349a5d9921056e9d86dc1a6357fb7fb"
RELATED="54ac629045a8d5b348f43f993894bbd9f0a707e6f165ff91d8e946ea41505810"


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
              "status":"ADAPTER_READS_COMPONENT_MODE_SOURCE; R138_XML_TO_MEMBER_UNBOUND",
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
    require(e["r138_job"]["job_xml_sha256"]=="fa65143f0bd7de52b36367137e61d681baa4c17824aded10be26317e7d1018a4"
            and e["typed_path"]["configuration_adapter_reads_member"]=="+0x54", "R138 profile and mode source")
    require(e["typed_path"]["xml_field_to_component_member_plus_0x54"]=="NOT_PROVEN"
            and e["typed_path"]["r138_component_mode"]=="UNBOUND"
            and e["typed_path"]["r138_custom_bsi_owner_u16"]=="UNBOUND", "open R138 edge")
    require(e["policy"]["new_dee_run"] is False and e["policy"]["key_or_license_content_read"] is False
            and e["policy"]["implementation_delivery"]=="REFERENCE_ONLY; NO_PRODUCT_RULE", "policy")
    sources=read_json("SOURCE-MANIFESTS.json")
    require(sources["incoming_package"]["manifest_sha256"]==INCOMING
            and sources["related_wash_package"]["manifest_sha256"]==RELATED, "source chain")
    verification=read_json("WASH-VERIFICATION.json")
    require(verification["incoming_manifest_sha256"]==INCOMING
            and verification["implementation_delivery"]=="REFERENCE_ONLY; NO_PRODUCT_RULE; implementation_ready=false", "verification")
    print(json.dumps({"status":"VALID_PARTIAL","package_id":PACKAGE_ID,"manifest_sha256":sha256(PACKAGE/"MANIFEST.json"),"r138_mode":"UNBOUND"},sort_keys=True))


if __name__=="__main__":
    if sys.argv[1:]==["--build-manifest"]:
        build_manifest()
    else:
        verify()
