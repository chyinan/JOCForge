# Composite child-open metadata owner boundary

The selected caller passes an input descriptor into the CompositeMaster item builder. A per-item child-open handle supplies a metadata result pair; the metadata view is copied into the source-record vector's second qword and passed to the composite builder.

The physical ADM N path is separate: incoming descriptor U16 +0x0A moves through open configuration and is stored as U32 N in a layout object. The inspected caller does not pass that scalar or layout object to the child-open call. No typed owner/pointer edge joins the child-open metadata view or host handle to the physical-N layout owner. Same-offset fields do not establish identity. Runtime callback and values remain NOT_CAPTURED.
