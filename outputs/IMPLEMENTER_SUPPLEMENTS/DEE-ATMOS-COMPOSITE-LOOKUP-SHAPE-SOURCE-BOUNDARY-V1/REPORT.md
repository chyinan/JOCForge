# Alternate composite shape metadata path

The inspected nonzero-layout path obtains a metadata pair from a lookup result or from a fallback object built with the selected source row's two qwords. Its shape interface reads a U16 count and six-byte entries from the per-source metadata view. The wrapper writes that count into the returned metadata view's count field.

The separate physical ADM N path reads descriptor U16 +0x0A, transfers it through open configuration, and stores U32 N in a layout object under a static 1..128 guard. No typed pointer/owner edge joins this physical-N field to the composition source-row vector or metadata-view pointer in the inspected path. The alternate branch closes the immediate shape count source; ultimate same-ADM-N identity and runtime branch remain UNBOUND/NOT_CAPTURED. No end-to-end range is established.
