"""Train an additive four-window speech classifier on independently synthesized audio."""
import argparse
import hashlib
import json
from pathlib import Path

import numpy as np
from sklearn.svm import LinearSVC


def read_scenes(directory, compact):
    manifest = directory / "manifest.json"
    groups = {"train": [], "validation": []}
    entries = json.loads(manifest.read_text(encoding="utf8"))["scenes"]
    if not compact:
        entries = sorted(entries, key=lambda item: item["id"])
    for scene in entries:
        content = (directory / scene["file"]).read_bytes()
        if hashlib.sha256(content).hexdigest() != scene["sha256"]:
            raise ValueError(f"Checksum mismatch: {scene['id']}")
        rows = json.loads(content)["examples"]
        x = np.asarray([row["x"] for row in rows]).reshape(-1, 3, 7)
        y = np.asarray([row["accepted"] for row in rows]).reshape(-1, 3)
        groups[scene["split"]].append((x, y))
    return groups, hashlib.sha256(manifest.read_bytes()).hexdigest()


def expand(scenes, cuts):
    xs, ys = [], []
    for x, y in scenes:
        tests = [x[:, :, j, None] >= values for j, values in enumerate(cuts)]
        basis = np.concatenate([np.ones((*x.shape[:2], 1)), *tests], axis=2)
        recent = basis.copy()
        for lag in (1, 2, 3):
            recent[lag:] += basis[:-lag]
        xs.append(recent.reshape(y.size, -1))
        ys.append(y.ravel())
    return np.concatenate(xs), np.concatenate(ys)


def train(directory, compact=False, frozen=False, expanded=False):
    scenes, digest = read_scenes(directory, compact)
    training_rows = np.concatenate([x.reshape(-1, 7) for x, _ in scenes["train"]])
    winner = None
    best = (np.inf, np.inf)
    grids = (16, 32, 64, 128, 256)
    regularizations = (0.1, 1.0, 10.0)
    iterations, tolerance = 4000, 1e-4
    if compact:
        regularizations = (0.01, 0.1, 1.0, 10.0)
        iterations, tolerance = 5000, 1e-5
    if expanded:
        grids, regularizations = (32, 64), (0.05, 0.1, 0.3, 1.0)
        iterations, tolerance = 3000, 1e-4
    if frozen:
        grids, regularizations = (32,), (0.1,)
    for grid in grids:
        cuts = [np.unique(np.quantile(axis, np.arange(1, grid) / grid)) for axis in training_rows.T]
        if compact:
            cuts = [axis.astype(np.float32).astype(np.float64) for axis in cuts]
        x_train, y_train = expand(scenes["train"], cuts)
        x_valid, y_valid = expand(scenes["validation"], cuts)
        for c in regularizations:
            fit = LinearSVC(C=c, penalty="l1", loss="squared_hinge", dual=False, fit_intercept=False,
                            random_state=43 if compact else 1, max_iter=iterations,
                            tol=tolerance).fit(x_train, y_train)
            weights = fit.coef_[0]
            errors = int(np.sum((x_valid @ weights >= 0) != y_valid))
            magnitude_floor = 1e-7 if compact else 0.0
            terms = int(np.count_nonzero(np.abs(weights) > magnitude_floor))
            if (errors, terms) >= best:
                continue
            best = errors, terms
            rules = []
            position = 1
            for feature, axis in enumerate(cuts):
                for threshold, coefficient in zip(axis, weights[position:position + len(axis)]):
                    if abs(coefficient) > magnitude_floor:
                        rules.append({"feature": feature + 1, "threshold": threshold, "contribution": coefficient})
                position += len(axis)
            winner = {
                "training_manifest_sha256": digest,
                "single_score": "bias plus contributions whose feature is >= threshold (features numbered 1 through 7)",
                "decision": "sum of last four same-lane single scores >= 0, zero fill before stream start",
                "bias": weights[0],
                "rules": rules,
                "grid_size": grid,
                    "threshold_precision": "binary32" if compact else "binary64",
                "C": c,
                "iterations": int(fit.n_iter_),
                "validation_errors": errors,
                "validation_rows": y_valid.size,
                "nonzero_including_bias": terms,
            }
    return winner


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("training", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("--compact", action="store_true", help="Use single-precision thresholds and the compact search configuration")
    parser.add_argument("--frozen", action="store_true", help="Refit the selected compact configuration (32 quantiles, C=0.1)")
    parser.add_argument("--expanded", action="store_true", help="Select a compact model for the 51-scene data with its original training grid")
    args = parser.parse_args()
    if args.frozen and not args.compact:
        parser.error("--frozen requires --compact")
    if args.expanded and (not args.compact or args.frozen):
        parser.error("--expanded requires --compact and the full selection run")
    result = train(args.training, args.compact, args.frozen, args.expanded)
    if not args.compact:
        # Full-precision parent artifacts predate this optional metadata field.
        result.pop("threshold_precision", None)
    # Both parent model hashes identify the original UTF-8, CRLF JSON artifacts.
    args.output.write_text(json.dumps(result, indent=2) + "\n", encoding="utf8", newline="\r\n")
    print(result["validation_errors"], "errors in", result["validation_rows"], "validation rows",
          result["nonzero_including_bias"], "nonzero coefficients")
