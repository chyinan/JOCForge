# Higher-invocation shared-root boundary

Native and static evidence reports writer and serializer/consumer as sibling stages in the same higher-caller frame, using the same root argument; writer executes first. Their local descriptors differ, so descriptor equality is not established.

The static caller shifts the next element of two root-owned history arrays into current and clears their tails before writer. After serializer commit, it writes the current heads of both arrays into one common output record. This binds common higher invocation, root owner, and shift-to-common-record structure.

This is not a consumer-body-to-writer data-flow receipt. The writer precedes consumer in the same invocation; consumer_N to writer_N is ruled out. A next-invocation relation remains unbound. The common record may contain control/history scalars and is not an access-unit identity token. Transport AU and source-generation identities remain UNBOUND.

The smallest follow-up is one non-payload history element receipt across shift, following writer entry, and common output record. Stop if it does not carry consumer-output identity. This package defines no production encoding rule.
