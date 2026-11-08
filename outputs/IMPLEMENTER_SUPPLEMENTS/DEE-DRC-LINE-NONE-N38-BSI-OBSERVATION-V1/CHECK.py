from pathlib import Path
import hashlib,json
ROOT=Path(__file__).resolve().parent
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def need(ok,msg):
 if not ok:raise SystemExit("FAIL: "+msg)
m=json.loads((ROOT/"MANIFEST.json").read_text(encoding="utf-8"));s=json.loads((ROOT/"package_seal.json").read_text(encoding="utf-8"))
r=json.loads((ROOT/"RESULTS.json").read_text(encoding="utf-8"))
for x in m["payloads"]:
 p=ROOT/x["file"];need(p.is_file() and p.stat().st_size==x["bytes"] and sha(p)==x["sha256"],"payload "+x["file"])
sealed={x["file"]:x for x in s["files"]}
for x in m["payloads"]:need(x["file"] in sealed and sealed[x["file"]]["sha256"]==x["sha256"],"seal "+x["file"])
need(sha(ROOT/"MANIFEST.json")==sealed["MANIFEST.json"]["sha256"],"manifest seal")
a,b=r["arms"]
need(a["line_mode_drc_profile"]=="film_light" and b["line_mode_drc_profile"]=="none","profile arms")
need(a["frame_count"]==32 and b["frame_count"]==32 and a["audio_block_count"]==192 and b["audio_block_count"]==192,"shape")
need(a["compre_sequence_sha256"]==b["compre_sequence_sha256"] and a["compre_code_counts"]==b["compre_code_counts"],"compre comparison")
need(a["dynrng_sequence_sha256"]!=b["dynrng_sequence_sha256"],"dynrng comparison")
need(b["dynrng_code_counts"]=={"0x00":192},"none observation")
need(r["comparison"]["ddp_parameter_0x0d_typed_join"]=="UNBOUND","typed join")
need(r["comparison"]["decoded_audio_gain_or_pcm_effect"]=="NOT_CAPTURED","audio effect")
need(r["probe_processing"]["encoded_output_read_in_memory_by_probe"] is True and r["probe_processing"]["raw_ec3_or_bsi_bytes_persisted_or_printed"] is False,"probe scope")
print(json.dumps({"status":"PASS","line_mode":"OBSERVED_BSI_DYNRNG_CHANGE_N38","audio_effect":"NOT_CAPTURED"}))
