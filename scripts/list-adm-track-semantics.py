# pattern: Imperative Shell
"""Extract static ADM track/channel/object semantics without loading BW64 PCM."""

from __future__ import annotations

import argparse
import csv
import json
import struct
import xml.etree.ElementTree as ET
from pathlib import Path


def local(tag: str) -> str:
    return tag.rsplit("}", 1)[-1]


def chunks(path: Path) -> dict[bytes, bytes]:
    result: dict[bytes, bytes] = {}
    with path.open("rb") as handle:
        header = handle.read(12)
        if len(header) != 12 or header[0:4] not in (b"BW64", b"RF64", b"RIFF") or header[8:12] != b"WAVE":
            raise ValueError("source is not BW64/RF64/RIFF WAVE")
        cursor = 12
        data_size: int | None = None
        while True:
            handle.seek(cursor)
            chunk_header = handle.read(8)
            if not chunk_header:
                break
            if len(chunk_header) != 8:
                raise ValueError("source contains a truncated chunk header")
            chunk_id = chunk_header[:4]
            declared = struct.unpack_from("<I", chunk_header, 4)[0]
            body_size = declared
            if chunk_id == b"ds64":
                handle.seek(cursor + 8)
                body = handle.read(max(declared, 24))
                if len(body) < 24:
                    raise ValueError("source ds64 is truncated")
                data_size = struct.unpack_from("<Q", body, 8)[0]
                body_size = declared
            elif chunk_id == b"data" and declared == 0xFFFFFFFF:
                if data_size is None:
                    raise ValueError("data appears before ds64")
                body_size = data_size
            if chunk_id in (b"axml", b"chna", b"fmt "):
                handle.seek(cursor + 8)
                result[chunk_id] = handle.read(body_size)
                if len(result[chunk_id]) != body_size:
                    raise ValueError(f"truncated {chunk_id!r} chunk")
            cursor += 8 + body_size + body_size % 2
    return result


def text_child(element: ET.Element, name: str) -> str:
    for child in element.iter():
        if local(child.tag) == name and child is not element:
            return (child.text or "").strip()
    return ""


def extract(path: Path) -> list[dict[str, object]]:
    raw = chunks(path)
    chna = raw.get(b"chna")
    axml = raw.get(b"axml")
    if chna is None or axml is None or len(chna) < 4:
        raise ValueError("source is missing chna/axml")
    track_count, uid_count = struct.unpack_from("<HH", chna)
    if track_count != uid_count or len(chna) < 4 + track_count * 40:
        raise ValueError("source CHNA counts are inconsistent")
    uid_to_index: dict[str, int] = {}
    for offset in range(4, 4 + track_count * 40, 40):
        index = struct.unpack_from("<H", chna, offset)[0]
        uid = chna[offset + 2 : offset + 14].split(b"\0", 1)[0].decode("ascii", "replace")
        uid_to_index[uid] = index
    root = ET.fromstring(axml)
    channels: dict[str, dict[str, object]] = {}
    for element in root.iter():
        if local(element.tag) != "audioChannelFormat":
            continue
        channel_id = element.attrib.get("audioChannelFormatID", "")
        block = next((child for child in element if local(child.tag) == "audioBlockFormat"), None)
        if not channel_id or block is None:
            continue
        values: dict[str, str] = {}
        for position in block.iter():
            if local(position.tag) == "position":
                coordinate = position.attrib.get("coordinate", "").lower()
                values[coordinate] = (position.text or position.attrib.get("value", "")).strip()
        channels[channel_id] = {
            "channel_name": element.attrib.get("audioChannelFormatName", ""),
            "speaker_label": text_child(element, "speakerLabel"),
            "azimuth": values.get("azimuth", values.get("x", "")),
            "elevation": values.get("elevation", values.get("y", "")),
            "distance": values.get("distance", values.get("z", "")),
        }
    packs: dict[str, list[str]] = {}
    for element in root.iter():
        if local(element.tag) == "audioPackFormat":
            pack_id = element.attrib.get("audioPackFormatID", "")
            packs[pack_id] = [
                (child.text or "").strip()
                for child in element
                if local(child.tag) == "audioChannelFormatIDRef"
            ]
    rows: list[dict[str, object]] = []
    for element in root.iter():
        if local(element.tag) != "audioObject":
            continue
        object_id = element.attrib.get("audioObjectID", "")
        object_name = element.attrib.get("audioObjectName", "")
        pack_ref = text_child(element, "audioPackFormatIDRef")
        channel_ids = packs.get(pack_ref, [])
        uid_refs = [
            (child.text or "").strip()
            for child in element
            if local(child.tag) == "audioTrackUIDRef"
        ]
        if len(channel_ids) != len(uid_refs):
            continue
        for channel_id, uid in zip(channel_ids, uid_refs):
            semantics = channels.get(channel_id, {})
            rows.append(
                {
                    "track_index": uid_to_index.get(uid, -1),
                    "object_id": object_id,
                    "object_name": object_name,
                    "pack_format_id": pack_ref,
                    "channel_id": channel_id,
                    "channel_name": semantics.get("channel_name", ""),
                    "speaker_label": semantics.get("speaker_label", ""),
                    "azimuth": semantics.get("azimuth", ""),
                    "elevation": semantics.get("elevation", ""),
                    "distance": semantics.get("distance", ""),
                    "track_uid": uid,
                }
            )
    rows.sort(key=lambda row: int(row["track_index"]))
    return rows


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("source", type=Path)
    parser.add_argument("root", type=Path)
    args = parser.parse_args()
    rows = extract(args.source)
    args.root.mkdir(parents=True, exist_ok=True)
    (args.root / "adm-track-semantics.json").write_text(json.dumps(rows, indent=2), encoding="utf-8")
    with (args.root / "adm-track-semantics.csv").open("w", newline="", encoding="utf-8") as handle:
        writer = csv.DictWriter(handle, fieldnames=rows[0].keys())
        writer.writeheader()
        writer.writerows(rows)
    print("track_rows", len(rows))
    for row in rows:
        print(row["track_index"], row["object_id"], row["object_name"], row["channel_name"], row["speaker_label"], row["azimuth"], row["elevation"], row["distance"])
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
