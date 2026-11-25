// pattern: Imperative Shell
// Compare the two public input modes using the same valid multiframe 48 kHz ADM.
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { residentImports } from './wasm-resident-imports.mjs';

const [wasmPath, admPath] = process.argv.slice(2);
if (!wasmPath || !admPath) throw new Error('usage: node scripts/test-wasm-streamed-topology.mjs MODULE.wasm INPUT.wav');
const module = await WebAssembly.compile(await readFile(wasmPath));
const adm = await readFile(admPath);

async function encode(streamed, layout, dependentCount) {
  let api;
  const imports = streamed ? { env: {
    jocforge_host_len: () => BigInt(adm.length),
    jocforge_host_read(offset, len, dst) {
      const start = Number(offset);
      if (start + len > adm.length) return -1;
      new Uint8Array(api.memory.buffer, dst, len).set(adm.subarray(start, start + len));
      return 0;
    },
  } } : residentImports;
  api = (await WebAssembly.instantiate(module, imports)).exports;
  const handle = api.jocforge_wasm_encoder_create();
  assert.notEqual(handle, 0);
  try {
    api.jocforge_wasm_encoder_set_layout(handle, layout);
    if (dependentCount !== null) api.jocforge_wasm_encoder_set_dependent_count(handle, dependentCount);
    if (!streamed) {
      for (let start = 0; start < adm.length; start += 65536) {
        const part = adm.subarray(start, start + 65536);
        const ptr = api.jocforge_wasm_alloc(part.length);
        assert.notEqual(ptr, 0);
        new Uint8Array(api.memory.buffer, ptr, part.length).set(part);
        const status = api.jocforge_wasm_encoder_push_bytes(handle, ptr, part.length);
        api.jocforge_wasm_dealloc(ptr, part.length);
        assert.equal(status, 0);
      }
    }
    const status = streamed ? api.jocforge_wasm_encoder_finish_streamed(handle) : api.jocforge_wasm_encoder_finish(handle);
    assert.equal(status, 1, 'expected an encoded bitstream');
    return Buffer.from(new Uint8Array(api.memory.buffer,
      api.jocforge_wasm_encoder_bitstream_ptr(handle), api.jocforge_wasm_encoder_bitstream_len(handle)));
  } finally {
    api.jocforge_wasm_encoder_destroy(handle);
  }
}

function frameTypes(bytes) {
  const types = [];
  for (let offset = 0; offset < bytes.length;) {
    assert.equal(bytes.readUInt16BE(offset), 0x0b77);
    const bsid = bytes[offset + 5] >>> 3;
    if (bsid === 6) {
      // The native Flat7x core uses 48 kHz / 640 kbps (2560 bytes for 1536 samples).
      assert.equal(bytes[offset + 4] >>> 1, 18);
      types.push('AC-3 I0');
      offset += 2560;
    } else {
      assert.equal(bsid, 16, 'expected an E-AC-3 syncframe');
      const header = bytes.readUInt16BE(offset + 2);
      types.push([header >>> 14, (header >>> 11) & 7]);
      offset += 2 * ((header & 0x7ff) + 1);
    }
    assert.ok(offset <= bytes.length, 'truncated syncframe');
  }
  return types;
}

// A non-default Flat7x composition must expose I0/D0/D1, not the intermediate AC-3 I0/D0 pair.
for (const [layout, dependentCount] of [[0, null], [0, 1], [1, null], [1, 2]]) {
  const resident = await encode(false, layout, dependentCount);
  const types = frameTypes(resident);
  const count = types.filter(frame => frame === 'AC-3 I0' || frame[0] === 0).length;
  assert.ok(count >= 2, 'provide at least two access units');
  const expected = layout === 1 && dependentCount === null
    ? ['AC-3 I0', [1, 0]]
    : [[0, 0], ...Array.from({ length: dependentCount ?? 0 }, (_, id) => [1, id])];
  assert.deepEqual(types, Array.from({ length: count }, () => expected).flat());
  const streamed = await encode(true, layout, dependentCount);
  assert.ok(streamed.equals(resident), `streamed bytes differ from resident output (layout=${layout}, dependent count=${dependentCount})`);
}
console.log('wasm host/resident topology: PASS');
