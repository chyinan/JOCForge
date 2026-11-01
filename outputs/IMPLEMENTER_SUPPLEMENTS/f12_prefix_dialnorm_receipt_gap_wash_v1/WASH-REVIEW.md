# Wash review: F06ND/F12 dialnorm receipt edge

The source package reports no verified typed finalized-programme identity receipt consumed by both F06ND and F12. The inspected clean model is not in its declared freeze manifest, is marked promoted but not independently reviewed, and represents measurements as an in-state dictionary/content digest plus event-name strings rather than typed consumer calls.

The approved specs define a programme measurement/dialnorm value and require the BSI/ID1 projections to carry a digest, but the F06ND interface names scalar dialnorm and source generation; the F12 wire field itself carries no programme identity. No shared typed F06ND/F12 receipt field is specified in this evidence.

Status remains NOT_READY / UNBOUND. This is not an implementation-ready product API requirement and must not be turned into a new receipt schema or a blocker for supported audio-core implementation. No native/DLL execution or media payload access occurred.
