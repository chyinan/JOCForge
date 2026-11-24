// pattern: Imperative Shell
// Exercise the wasm32 encode ABI: allocation ownership and rejected input.
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { residentImports } from './wasm-resident-imports.mjs';

const [wasmPath] = process.argv.slice(2);
assert(wasmPath, 'usage: node scripts/test-wasm.mjs MODULE.wasm');
const { instance } = await WebAssembly.instantiate(await readFile(wasmPath), residentImports);
const api = instance.exports;

function withBytes(bytes, action) {
  const pointer = api.jocforge_wasm_alloc(bytes.length);
  assert(pointer !== 0, 'bounded allocation succeeds');
  try {
    new Uint8Array(api.memory.buffer, pointer, bytes.length).set(bytes);
    return action(pointer, bytes.length);
  } finally {
    api.jocforge_wasm_dealloc(pointer, bytes.length);
  }
}

assert.equal(api.jocforge_wasm_encoder_finish(0), -1);
assert.equal(api.jocforge_wasm_alloc(64 * 1024 * 1024 + 1), 0);
const handle = api.jocforge_wasm_encoder_create();
assert(handle !== 0, 'encoder constructor');
assert.equal(api.jocforge_wasm_encoder_push_bytes(handle, 0x1000, 64), -1);
const status = withBytes(Buffer.from('RIFF'), (pointer, length) =>
  api.jocforge_wasm_encoder_push_bytes(handle, pointer, length));
assert.equal(status, 0, 'partial input is retained');
assert.equal(api.jocforge_wasm_encoder_finish(handle), -1, 'truncated RIFF is rejected');
assert.equal(api.jocforge_wasm_encoder_error_category(handle), 0);
api.jocforge_wasm_encoder_destroy(handle);
assert.equal(api.jocforge_wasm_encoder_finish(handle), -1, 'destroyed handle rejected');
// A host with no OPFS file can instantiate the same module; streaming fails as input, not linkage.
const noFile = api.jocforge_wasm_encoder_create();
assert.notEqual(noFile, 0);
assert.equal(api.jocforge_wasm_encoder_finish_streamed(noFile), -1);
assert.equal(api.jocforge_wasm_encoder_error_category(noFile), 0);
api.jocforge_wasm_encoder_destroy(noFile);
console.log('wasm abi: PASS');
