// pattern: Functional Core
use jocforge_domain::timeline::{
    AdmSampleCount, ClockError, CodecSourceIndex, FrontendCallbackIndex, JocDispatchIndex,
    PublicationDisposition, programme_counts, publication_for_source, source_for_callback,
};

#[test]
fn sample_count_accounting_matches_scoped_contract() {
    let cases = [
        (1, (1, 1535, 5, 3, 1, 1)),
        (1535, (1, 1, 5, 3, 1, 1535)),
        (1536, (1, 0, 5, 3, 1, 1536)),
        (1537, (2, 1535, 6, 4, 2, 1)),
        (6_522_906, (4247, 486, 4251, 4249, 4247, 1050)),
    ];
    for (samples, expected) in cases {
        let actual = programme_counts(AdmSampleCount::new(samples).unwrap()).unwrap();
        assert_eq!(actual.as_tuple(), expected);
    }
    assert_eq!(AdmSampleCount::new(0), Err(ClockError::ZeroSampleCount));
    assert!(programme_counts(AdmSampleCount::new(u64::MAX).unwrap()).is_err());
}

#[test]
fn callback_source_dispatch_and_generated_writer_are_checked() {
    assert_eq!(source_for_callback(FrontendCallbackIndex::new(0)), None);
    assert_eq!(source_for_callback(FrontendCallbackIndex::new(1)), None);
    let source = source_for_callback(FrontendCallbackIndex::new(2)).unwrap();
    assert_eq!(source.value(), 0);
    assert_eq!(source.dispatch().unwrap().value(), 1);
    assert_eq!(
        source
            .dispatch()
            .unwrap()
            .generated_writer()
            .unwrap()
            .value(),
        0
    );
    let last = source_for_callback(FrontendCallbackIndex::new(4250)).unwrap();
    let last_dispatch = last.dispatch().unwrap();
    assert_eq!(
        (
            last.value(),
            last_dispatch.value(),
            last_dispatch.generated_writer().unwrap().value()
        ),
        (4248, 4249, 4248)
    );
}

#[test]
fn checked_conversions_distinguish_underflow_and_overflow() {
    let underflow = JocDispatchIndex::new(0).generated_writer().unwrap_err();
    assert_eq!(underflow, ClockError::ArithmeticUnderflow);
    assert_eq!(
        CodecSourceIndex::new(u64::MAX).dispatch(),
        Err(ClockError::ArithmeticOverflow)
    );
}

#[test]
fn publication_distinguishes_generated_popped_and_ordinary_writers() {
    assert!(matches!(
        publication_for_source(CodecSourceIndex::new(0)).unwrap(),
        PublicationDisposition::Suppressed { joc_popped: None }
    ));
    match publication_for_source(CodecSourceIndex::new(1)).unwrap() {
        PublicationDisposition::Suppressed {
            joc_popped: Some(writer),
        } => assert_eq!(writer.value(), 0),
        other @ PublicationDisposition::Suppressed { joc_popped: None } => {
            panic!("unexpected disposition: {other:?}")
        }
        other @ PublicationDisposition::Published(_) => panic!("unexpected disposition: {other:?}"),
    }
    match publication_for_source(CodecSourceIndex::new(2)).unwrap() {
        PublicationDisposition::Published(x) => assert_eq!(
            (x.au.value(), x.ordinary.value(), x.joc_popped.value()),
            (0, 0, 1)
        ),
        other @ PublicationDisposition::Suppressed { .. } => {
            panic!("unexpected disposition: {other:?}")
        }
    }
    match publication_for_source(CodecSourceIndex::new(4248)).unwrap() {
        PublicationDisposition::Published(x) => assert_eq!(
            (x.au.value(), x.ordinary.value(), x.joc_popped.value()),
            (4246, 4246, 4247)
        ),
        other @ PublicationDisposition::Suppressed { .. } => {
            panic!("unexpected disposition: {other:?}")
        }
    }
}

#[test]
fn terminal_pending_writers_are_explicit() {
    let counts = programme_counts(AdmSampleCount::new(6_522_906).unwrap()).unwrap();
    assert_eq!(
        counts
            .ordinary_pending()
            .unwrap()
            .map(jocforge_domain::timeline::OrdinaryPayloadWriterIndex::value),
        [4247, 4248]
    );
    assert_eq!(counts.joc_pending().unwrap().value(), 4248);
}
