"""Calibrate a fitted head's single-window bias to synthetic validation prevalence."""
import argparse
import hashlib
import json
from pathlib import Path

import numpy as np


def score(scene, model):
    rows = scene["examples"]
    features = np.asarray([row["x"] for row in rows]).reshape(-1, 3, 7)
    labels = np.asarray([row["accepted"] for row in rows]).reshape(-1, 3)
    single = np.full(labels.shape, model["bias"], dtype=np.float64)
    for rule in model["rules"]:
        single += rule["contribution"] * (features[:, :, rule["feature"] - 1] >= rule["threshold"])
    total = single.copy()
    for lag in (1, 2, 3):
        total[lag:] += single[:-lag]
    mean = total / np.minimum(np.arange(1, len(total) + 1), 4)[:, None]
    return mean.ravel(), labels.ravel()


def average_independent_heads(models, digests):
    values = {}
    for model in models:
        for rule in model["rules"]:
            key = (rule["feature"], rule["threshold"])
            values[key] = values.get(key, 0.0) + rule["contribution"] / len(models)
    return {
        "training_manifest_sha256": models[0]["training_manifest_sha256"],
        "component_sha256": digests,
        "single_score": "bias plus contributions whose feature is >= threshold (features numbered 1 through 7)",
        "decision": "sum of last four same-lane single scores >= 0, zero fill before stream start",
        "bias": sum(model["bias"] for model in models) / len(models),
        "rules": [{"feature": axis, "threshold": cut, "contribution": weight}
                  for (axis, cut), weight in sorted(values.items()) if weight],
    }


def calibrate(training, head, partner=None):
    manifest = (training / "manifest.json").read_bytes()
    paths = [head] if partner is None else [head, partner]
    hashes = [hashlib.sha256(path.read_bytes()).hexdigest() for path in paths]
    components = [json.loads(path.read_text(encoding="utf8")) for path in paths]
    if any(hashlib.sha256(manifest).hexdigest() != m["training_manifest_sha256"] for m in components):
        raise ValueError("Use the head's original training and validation scene manifest")
    model = components[0] if partner is None else average_independent_heads(components, hashes)
    scores, flags = [], []
    for scene in json.loads(manifest)["scenes"]:
        if scene["split"] == "validation":
            content = (training / scene["file"]).read_bytes()
            if hashlib.sha256(content).hexdigest() != scene["sha256"]:
                raise ValueError(f"Scene changed: {scene['id']}")
            material = json.loads(content)
            averages, labels = score(material, components[0])
            if partner is not None:
                other, _ = score(material, components[1])
                averages = (averages + other) * 0.5
            scores.extend(averages)
            flags.extend(labels)
    observed = np.asarray(flags)
    ordered = np.sort(scores)
    index = len(ordered) - int(observed.sum())
    if not 0 < index < len(ordered) or ordered[index - 1] == ordered[index]:
        raise ValueError("A single threshold cannot split this pair of calibration scores")
    correction = float((ordered[index - 1] + ordered[index]) / 2)
    raw = np.asarray(scores)
    stats = {"base_model_sha256": hashes[0] if partner is None else hashes,
             "base_bias": model["bias"], "subtracted": correction,
             "validation_target_positives": int(observed.sum()), "validation_rows": len(raw),
             "uncalibrated_positives": int(np.sum(raw >= 0)),
             "calibrated_positives": int(np.sum(raw >= correction)),
             "uncalibrated_errors": int(np.sum((raw >= 0) != observed)),
             "calibrated_errors": int(np.sum((raw >= correction) != observed))}
    model["bias"] -= correction
    model["base_validation_errors"] = stats["uncalibrated_errors"]
    model["validation_errors"] = stats["calibrated_errors"]
    model["prevalence_calibration"] = stats
    return model


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("training", type=Path)
    parser.add_argument("head", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("--average-with", type=Path, help="Give a second independent head equal weight before fitting the bias")
    args = parser.parse_args()
    result = calibrate(args.training, args.head, args.average_with)
    # Both parent model hashes identify the original UTF-8, CRLF JSON artifacts.
    args.output.write_text(json.dumps(result, indent=2) + "\n", encoding="utf8", newline="\r\n")
    print(result["prevalence_calibration"])
