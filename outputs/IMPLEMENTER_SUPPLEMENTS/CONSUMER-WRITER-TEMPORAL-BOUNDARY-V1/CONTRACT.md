# Consumer/writer temporal boundary

Native/static boundary observation only; not an access-unit or source-generation contract.

Within one higher invocation, the reported order is: shift two root-owned per-generation arrays and clear their new tail; execute the writer sibling stage; then execute the serializer/consumer sibling stage. The writer and consumer share the higher invocation, but the writer is not inside the serializer/consumer function. The serializer-consumer entry counter is not a writer-generation token.

In one bounded consumer observation, start_index is 2 and the six per-lane counts are [0,0,457,0,0,0], giving 457 consumed bytes. After consumer, per-lane CRC words are appended and finalized per-lane buffers are copied to a separate contiguous destination. The observed 2048-word copy source and writer's first five arguments had no direct alias to the consumer body/state.

Consequently consumer_N to writer_N is ruled out by stage order. consumer_N to writer_N+1 remains possible through the root-state shift but is unbound. Same-AU, writer-output, and source-generation identities remain UNBOUND. The next minimum evidence is one before/after receipt for a root-array element across the higher-invocation shift, compared with the writer-stage result or following contiguous destination.
