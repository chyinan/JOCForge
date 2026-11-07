# DEE time-base generic host setter boundary

The bounded generic host SetProperty adapter copies a property record vector and ends at an indirect call through proxy vtable +0x08. Its inspected body has no explicit operand for the DDP option descriptor at +0x550. Eight bounded direct callsites carry source/target routing properties; they do not show time_base, start, or end values.

The selected DDP proxy instance and its concrete +0x08 target remain unbound. The minimum next edge is to bind that selected proxy, then inspect only its setter for a DDP descriptor write. This package does not prove a whole-host absence or a runtime value.
