// pattern: Imperative Shell
// Drive the built wasm32 module over the exported byte ABI and compare its bitstream with the
// native product produced from the same excerpt.
import { readFile, writeFile } from 'node:fs/promises';
import { residentImports } from './wasm-resident-imports.mjs';

const [wasmPath, excerptPath, nativePath, outPath] = process.argv.slice(2);
if (!wasmPath || !excerptPath || !nativePath || !outPath) {
  throw new Error('usage: node scripts/compare-wasm-encode.mjs MODULE.wasm EXCERPT.wav NATIVE.ec3 OUT.ec3');
}

const { instance } = await WebAssembly.instantiate(await readFile(wasmPath), residentImports);
const api = instance.exports;
const input = await readFile(excerptPath);

const handle = api.jocforge_wasm_encoder_create();
if (handle === 0) throw new Error('encoder handle was not created');
const pointer = api.jocforge_wasm_alloc(input.length);
if (pointer === 0) throw new Error('input allocation was refused');
new Uint8Array(api.memory.buffer, pointer, input.length).set(input);

const pushStatus = api.jocforge_wasm_encoder_push_bytes(handle, pointer, input.length);
api.jocforge_wasm_dealloc(pointer, input.length);
const finishStatus = api.jocforge_wasm_encoder_finish(handle);
if (finishStatus < 0) {
  const category = api.jocforge_wasm_encoder_error_category(handle);
  const errorPointer = api.jocforge_wasm_encoder_error_ptr(handle);
  const errorLength = api.jocforge_wasm_encoder_error_len(handle);
  const detail = errorPointer === 0
    ? ''
    : Buffer.from(new Uint8Array(api.memory.buffer, errorPointer, errorLength)).toString('utf8');
  throw new Error(`wasm encode failed: category=${category} detail=${detail}`);
}

const bitstreamPointer = api.jocforge_wasm_encoder_bitstream_ptr(handle);
const bitstreamLength = api.jocforge_wasm_encoder_bitstream_len(handle);
if (bitstreamPointer === 0 || bitstreamLength === 0) {
  throw new Error('wasm encode produced no bitstream');
}
const product = Buffer.from(new Uint8Array(api.memory.buffer, bitstreamPointer, bitstreamLength));
api.jocforge_wasm_encoder_destroy(handle);

await writeFile(outPath, product);
const native = await readFile(nativePath);
const equal = product.length === native.length && Buffer.compare(product, native) === 0;
console.log(`wasm push=${pushStatus} finish=${finishStatus}`);
console.log(`excerpt=${input.length} wasm=${product.length} native=${native.length} bit_exact=${equal}`);
if (!equal) {
  const limit = Math.min(product.length, native.length);
  let first = -1;
  for (let index = 0; index < limit; index += 1) {
    if (product[index] !== native[index]) { first = index; break; }
  }
  throw new Error(`bitstream differs, first mismatch at byte ${first}`);
}
