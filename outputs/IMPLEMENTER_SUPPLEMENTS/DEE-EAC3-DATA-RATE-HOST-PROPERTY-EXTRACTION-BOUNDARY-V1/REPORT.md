# Host data_rate property extraction boundary

## Disposition

GENERIC_XML_STRING_EXTRACTION_BOUND; DDP_DATA_RATE_PROPERTY_OWNER_UNBOUND; STRING_TO_I32_UNBOUND; PROVIDER_DISPATCH_UNBOUND; RUNTIME_NOT_RUN.

The bounded host path is FUN_14002a0b0 -> dee::xml::XmlParse::Parse -> FUN_140029330 -> FUN_1400207f0. FUN_140029330 reads a string-like member at param_1 +0x68 and passes it to a generic string-copy helper. The inspected helper performs bounds/copy/null-termination logic; it does not identify the DDP data_rate member or convert it to a signed-I32 selector.

The host strings describing data_rate values occur in FUN_1400b9d20..FUN_1400c3698 but are not joined to the parsed XML node or provider request. The plugin-side open-request field FUN_180178dc0.param_1[0] is a signed I32, but the route-specific host property owner that writes it remains unbound.

## Independent Wash checks

Wash recomputed the source manifest, all nine listed files, MANIFEST.sha256, six parent manifests, and current host/plugin binary hashes; all matched. The source package has no formal seal. Its checker claims normal and python -O success; Wash did not execute it. Raw trace/script/checker files are omitted from this supplement.

No DEE or encoding run, media/payload/key/license access, or implementation contact occurred.

## Implementation boundary

The public XML data_rate option is not yet connected to the plugin selector. Do not implement a selector mapping from enum order or matching numeric text. The exact missing edge is the host property handler that reads /encode_to_atmos_ddp/data_rate and writes the provider's signed-I32 request field.
