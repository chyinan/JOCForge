from __future__ import annotations
import hashlib,json,re
from pathlib import Path
ROOT=Path(__file__).resolve().parent
def require(condition:bool,message:str)->None:
    if not condition: raise SystemExit(message)
raw=(ROOT/"MANIFEST.json").read_bytes()
expected=(ROOT/"MANIFEST.sha256").read_text(encoding="ascii").split()[0]
require(hashlib.sha256(raw).hexdigest()==expected,"manifest hash mismatch")
m=json.loads(raw)
require(m["schema"]=="washed-r138-replay-helper-identity-crosscheck.v1","schema mismatch")
require(m["package_id"]=="r138_replay_helper_identity_crosscheck_v1","package mismatch")
require(m["status"]=="R138_EQUIVALENT_REPLAY_HOST_TO_HELPER_IMAGE_BOUND; ORIGINAL_R138_CHILD_RECEIPT_NOT_CAPTURED","status mismatch")
require(m["implementation_readiness"]=="CANDIDATE_HELPER_IDENTITY_CONFIRMED_FOR_R138_EQUIVALENT_REPLAY_ONLY","scope mismatch")
require(m["audio_algorithm_inference_allowed"] is False,"algorithm scope widened")
for e in m["payloads"]:
 p=(ROOT/e["file"]).resolve()
 require(p.is_relative_to(ROOT.resolve()) and p.is_file(),"unsafe or missing payload")
 d=p.read_bytes()
 require(len(d)==e["bytes"] and hashlib.sha256(d).hexdigest()==e["sha256"],"payload hash mismatch")
o=json.loads((ROOT/"OBSERVATION.json").read_text(encoding="utf-8"))
require(o["host_process"]["matches_r138_host_image"] is True,"host identity mismatch")
require(o["helper_child"]["sha256_at_resolved_child_path"]==o["helper_child"]["helper_candidate_sha256"],"helper SHA mismatch")
require(o["helper_child"]["child_create_observed_same_debug_session"] is True,"child creation not joined")
require(o["scope"]["input_sha256"]=="7826dcbef37ef69642a6164c97a37bcfb3f917cea74aa0f0a30d76d0a825f1d1","input hash mismatch")
require(o["temporary_output"]["parsed_or_played"] is False and o["temporary_output"]["removed_after_hash"] is True,"output policy mismatch")
require(o["helper_rule_context"]["original_r138_child_pid_receipt"]=="NOT_CAPTURED","historical PID receipt promoted")
for name in ("WASH-REVIEW.md","OBSERVATION.json","VERIFICATION.json"):
 t=(ROOT/name).read_text(encoding="utf-8")
 require("D:\\" not in t and "C:\\" not in t,"absolute path leaked")
 require("FUN_180" not in t,"native symbol leaked")
 require("RVA" not in t,"native coordinate leaked")
 require("-----BEGIN" not in t,"key-like material leaked")
print("R138_EQUIVALENT_HELPER_IDENTITY=PASS")
print("ORIGINAL_R138_CHILD_RECEIPT=NOT_CAPTURED")
