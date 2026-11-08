from pathlib import Path
import hashlib,json
ROOT=Path(__file__).resolve().parent
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def need(ok,msg):
 if not ok:raise SystemExit("FAIL: "+msg)
m=json.loads((ROOT/"MANIFEST.json").read_text(encoding="utf-8"));s=json.loads((ROOT/"package_seal.json").read_text(encoding="utf-8"))
r=json.loads((ROOT/"RESULTS.json").read_text(encoding="utf-8"));b=json.loads((ROOT/"SOURCE-BINDING.json").read_text(encoding="utf-8"))
for x in m["payloads"]:
 p=ROOT/x["file"];need(p.is_file() and p.stat().st_size==x["bytes"] and sha(p)==x["sha256"],"payload "+x["file"])
seal={x["file"]:x for x in s["files"]}
for x in m["payloads"]:need(x["file"] in seal and seal[x["file"]]["sha256"]==x["sha256"],"seal "+x["file"])
need(sha(ROOT/"MANIFEST.json")==seal["MANIFEST.json"]["sha256"],"manifest seal")
need(r["input"]["channels"]==38 and r["variable"]=="rf_mode_drc_profile","vector scope")
need(r["arms"][0]["dee_exit_code"]==0 and r["arms"][1]["dee_exit_code"]==0,"run exit")
need(r["result"]["bsi_compre_sequence_changed"] is True,"compre difference")
need(r["result"]["dynrng_sequence_same"] is True,"dynrng scope")
need(r["result"]["audio_pcm_or_decoded_effect"]=="NOT_CAPTURED","audio effect overclaim")
need(r["result"]["rf_profile_audio_gain_semantics"]=="UNBOUND","gain semantics")
need(b["payload_or_source_paths_included"] is False,"source path leakage")
need(all(not a["raw_bsi_or_frame_bytes_persisted"] for a in r["arms"]),"raw bytes persisted")
need(r["exclusions"]["pcm_read"] is False and r["exclusions"]["private_key_or_license_read"] is False,"payload/secret policy")
print(json.dumps({"status":"PASS","rf_bsi_effect":"OBSERVED_N38_PAIR","audio_effect":"NOT_CAPTURED","generalization":"NOT_ESTABLISHED"}))
