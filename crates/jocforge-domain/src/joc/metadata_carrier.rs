// pattern: Functional Core
//! Complete staged F13 EMDF container to selected carrier reservation and body bytes.

use crate::joc::emdf_carrier::{
    CarrierBitCursor, CarrierPlanReceipt, CarrierPlannerState, CarrierWriteError,
    SelectedCarrierAdmissionError, SerializedEmdfBodyFragment, write_body_fragment,
    write_fragment_reservation,
};

const EMDF_MARKER: [u8; 2] = [0x58, 0x38];
const PREFIX_BYTES: usize = 4;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MetadataCarrierError {
    ContainerShape,
    ContainerLength,
    Admission(SelectedCarrierAdmissionError),
    Write(CarrierWriteError),
}

#[must_use]
#[derive(Debug, Eq, PartialEq)]
pub struct MetadataCarrierProposal {
    planner_after: CarrierPlannerState,
    plan: CarrierPlanReceipt,
    frame: Box<[u8]>,
    reservation_cursor: CarrierBitCursor,
    payload_cursor: CarrierBitCursor,
    end_cursor: CarrierBitCursor,
    complete_container: Box<[u8]>,
}

impl MetadataCarrierProposal {
    pub const fn planner_after(&self) -> &CarrierPlannerState {
        &self.planner_after
    }

    pub const fn plan(&self) -> CarrierPlanReceipt {
        self.plan
    }

    pub fn frame(&self) -> &[u8] {
        &self.frame
    }

    pub const fn reservation_cursor(&self) -> CarrierBitCursor {
        self.reservation_cursor
    }

    pub const fn payload_cursor(&self) -> CarrierBitCursor {
        self.payload_cursor
    }

    pub const fn end_cursor(&self) -> CarrierBitCursor {
        self.end_cursor
    }

    pub fn complete_container(&self) -> &[u8] {
        &self.complete_container
    }

    pub fn into_parts(self) -> (CarrierPlannerState, Box<[u8]>) {
        (self.planner_after, self.frame)
    }
}

/// Stages one already-serialized F13 container at a caller-supplied F12 bit
/// cursor. This adapter does not derive frame syntax, the cursor, or AU output.
pub fn stage_metadata_carrier(
    planner_before: &CarrierPlannerState,
    complete_container: &[u8],
    frame_template: &[u8],
    reservation_cursor: CarrierBitCursor,
) -> Result<MetadataCarrierProposal, MetadataCarrierError> {
    if complete_container.len() < 5 || complete_container[..2] != EMDF_MARKER {
        return Err(MetadataCarrierError::ContainerShape);
    }
    let declared_body = usize::from(u16::from_be_bytes([
        complete_container[2],
        complete_container[3],
    ]));
    if declared_body != complete_container.len() - PREFIX_BYTES {
        return Err(MetadataCarrierError::ContainerLength);
    }
    let complete_length = u16::try_from(complete_container.len())
        .map_err(|_| MetadataCarrierError::ContainerLength)?;
    let body_length =
        u16::try_from(declared_body).map_err(|_| MetadataCarrierError::ContainerLength)?;
    let mut planner_after = planner_before.staged_copy();
    let plan = planner_after
        .plan_selected_container(Some(complete_length))
        .map_err(MetadataCarrierError::Admission)?;
    if !plan.enabled()
        || plan.return_i32() != 0
        || plan.first_block().value() != 2
        || plan.segment_lengths()[2] != body_length
        || plan.segment_lengths()[..2]
            .iter()
            .any(|length| *length != 0)
        || plan.segment_lengths()[3..]
            .iter()
            .any(|length| *length != 0)
    {
        return Err(MetadataCarrierError::ContainerLength);
    }

    let mut frame = frame_template.to_vec();
    let payload_cursor =
        write_fragment_reservation(&mut frame, reservation_cursor, true, complete_length)
            .map_err(MetadataCarrierError::Write)?;
    let end_cursor = write_body_fragment(
        &mut frame,
        payload_cursor,
        SerializedEmdfBodyFragment::new(&complete_container[PREFIX_BYTES..], body_length, true),
    )
    .map_err(MetadataCarrierError::Write)?;
    Ok(MetadataCarrierProposal {
        planner_after,
        plan,
        frame: frame.into_boxed_slice(),
        reservation_cursor,
        payload_cursor,
        end_cursor,
        complete_container: complete_container.to_vec().into_boxed_slice(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selected_complete_container_stages_at_explicit_unaligned_cursor() {
        let container = [0x58, 0x38, 0x00, 0x03, 0x12, 0x34, 0x56];
        let template = [0xa5_u8; 32];
        let cursor = CarrierBitCursor::new(11);
        let proposal =
            stage_metadata_carrier(&CarrierPlannerState::cold(), &container, &template, cursor)
                .unwrap();
        assert_eq!(proposal.plan().segment_lengths()[2], 3);
        assert_eq!(proposal.payload_cursor().bit_offset(), 21);
        assert_eq!(proposal.end_cursor().bit_offset(), 77);
        assert_eq!(proposal.complete_container(), container);
        assert_ne!(proposal.frame(), template);
    }

    #[test]
    fn malformed_container_is_rejected_without_touching_the_planner() {
        let planner = CarrierPlannerState::cold();
        assert_eq!(
            stage_metadata_carrier(
                &planner,
                &[0x58, 0x38, 0, 2, 0],
                &[0; 16],
                CarrierBitCursor::new(0),
            ),
            Err(MetadataCarrierError::ContainerLength)
        );
        assert_eq!(planner, CarrierPlannerState::cold());
    }
}
