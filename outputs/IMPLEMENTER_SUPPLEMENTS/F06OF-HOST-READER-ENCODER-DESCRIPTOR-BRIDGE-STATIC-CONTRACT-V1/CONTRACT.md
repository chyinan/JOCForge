# Host reader to encoder descriptor static bridge

Status: `READER_DESCRIPTOR_TO_PE_ENCODER_WRAPPER_STATIC_CLOSED; CONVERTER_SOURCE_JOIN_UNBOUND`.

## Conditional worker path

On the reviewed host worker path, a plain ADM reader returns a worker-local frame descriptor `D`. The same descriptor object is passed first to the reader's frame-read interface and immediately afterward to the public PE encoder process interface. The encoder process forwards `D` into its private processing path, whose lazy-open route reaches the profile-input wrapper described by the separate F13C wrapper contract.

The exact image-bound branches are conditional: the worker configuration selects the plain-reader branch and the PE encoder backend branch. Their values in the requested selected run are not captured. This does not bind another reader branch or the AtmosProcessor backend to the same path.

## Reader-owned descriptor metadata

The reader populates semantic fields on `D` from its reader-owned composite master: channel/frame metadata, an auxiliary metadata-query result, and a copied metadata-row vector. The encoder process consumes the same `D` object. The inspected receipt records the static field flow only; no runtime pointer values or pointed-to metadata were read.

The reader's frame descriptor and the F06OF measurement descriptor are separate interface types. Matching channel counts, frame sizes, or similarly named fields do not prove they are the same descriptor. This bridge therefore does not close the selected physical input-row/read result to meter `channel_count`/`frame_size` in the same source generation.

## Host source and converter ownership remain open

The host loads a source property through a plug-in proxy and dispatches processing stages through that proxy. The concrete proxy implementation that copies the selected source reference into the worker configuration is not bound in the reviewed static artifacts.

The encoder's storage system registers an ADM/BWF factory and has a source-item/master-open path. That encoder-side master is not identity-joined to the converter module's ADM/CHNA storage owner. The full relation `converter source C -> host source property -> worker configuration -> reader master D -> measurement descriptor` remains `UNBOUND`.

Keep selected host invocation, source/reset generation, physical-row-to-measurement-descriptor identity, EOF/drain, and converter-to-encoder source ownership open. No native run, audio read, implementation algorithm change, or product-ready claim follows from this static bridge.
