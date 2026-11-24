// pattern: Imperative Shell
// Compare the two public input modes using the same valid multiframe 48 kHz ADM.
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { residentImports } from './wasm-resident-imports.mjs';

const [wasmPath, admPath] = process.argv.slice(2);
if (!wasmPath || !admPath) throw new Error('usage: node scripts/test-wasm-streamed-topology.mjs MODULE.wasm INPUT.wav');
const module = await WebAssembly.compile(await readFile(wasmPath));
const adm = await readFile(admPath);

async function encode(streamed, composed) {
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
    if (composed) api.jocforge_wasm_encoder_set_dependent_count(handle, 1);
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
    const header = bytes.readUInt16BE(offset + 2);
    types.push(header >>> 14);
    offset += 2 * ((header & 0x7ff) + 1);
    assert.ok(offset <= bytes.length, 'truncated E-AC-3 syncframe');
  }
  return types;
}

for (const composed of [false, true]) {
  const resident = await encode(false, composed);
  const types = frameTypes(resident);
  const count = types.filter(type => type === 0).length;
  assert.ok(count >= 2, 'provide at least two access units');
  assert.deepEqual(types, Array.from({ length: count }, () => composed ? [0, 1] : [0]).flat());
  const streamed = await encode(true, composed);
  assert.ok(streamed.equals(resident), 'streamed bytes differ from resident output (dependent count: ' + (composed ? 1 : 'unset') + ')');
}
console.log('wasm host/resident topology: PASS');
