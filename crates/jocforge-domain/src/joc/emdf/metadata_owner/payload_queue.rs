use super::{MetadataOwnerError, MetadataQueueProducerProvenance, hex_encode};
use crate::{
    timeline::{CodecSourceIndex, JocDispatchIndex},
    transaction::CallbackTransactionId,
};
use serde_json::{Value, json};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct QueuedBody {
    pub writer_generation: CodecSourceIndex,
    pub payload: Vec<u8>,
    pub producer_provenance: Option<MetadataQueueProducerProvenance>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct PayloadQueue {
    pub id: u8,
    pub capacity: u16,
    pub delay: u32,
    pub frame: u16,
    pub write_slot: u16,
    pub slots: Vec<Option<QueuedBody>>,
    pub buffer_ids: Vec<u16>,
}

impl PayloadQueue {
    pub(super) fn cold(id: u8, capacity: u16, delay: u32, slot_count: usize) -> Self {
        Self {
            id,
            capacity,
            delay,
            frame: 1536,
            write_slot: 0,
            slots: vec![None; slot_count],
            buffer_ids: (0..slot_count as u16).collect(),
        }
    }

    pub fn push_then_pop(
        &mut self,
        body: &QueuedBody,
    ) -> Result<QueueOperation, MetadataOwnerError> {
        if body.payload.len() > usize::from(self.capacity) {
            return Err(MetadataOwnerError::Capacity);
        }
        let slot = usize::from(self.write_slot);
        if self.slots[slot].is_some() {
            return Err(MetadataOwnerError::QueueOccupied);
        }

        let before = self.operation_view();
        let mut pushed_slots = self.slots.clone();
        pushed_slots[slot] = Some(body.clone());
        let popped_index = pushed_slots.len() - 1;
        let popped = pushed_slots[popped_index].take();
        let popped_buffer_id = self.buffer_ids[popped_index];

        let mut rotated_slots = vec![None; pushed_slots.len()];
        rotated_slots[1..].clone_from_slice(&pushed_slots[..popped_index]);
        let mut rotated_ids = Vec::with_capacity(self.buffer_ids.len());
        rotated_ids.push(popped_buffer_id);
        rotated_ids.extend_from_slice(&self.buffer_ids[..popped_index]);
        self.slots = rotated_slots;
        self.buffer_ids = rotated_ids;

        Ok(QueueOperation {
            queue_id: self.id,
            pushed_writer: body.writer_generation,
            pushed_bytes: body.payload.len() as u16,
            pushed_provenance: body.producer_provenance.clone(),
            consumer_transaction_id: None,
            consumer_source_index: None,
            consumer_dispatch: None,
            popped_writer: popped.as_ref().map(|entry| entry.writer_generation),
            popped_buffer_id,
            before,
            after: self.operation_view(),
            popped,
        })
    }

    pub fn state_json(&self) -> Value {
        let slots = self
            .slots
            .iter()
            .map(|slot| {
                slot.as_ref().map_or(Value::Null, |body| {
                    let mut value = json!({
                        "writer_generation": body.writer_generation.value(),
                        "payload_hex": hex_encode(&body.payload),
                    });
                    if let Some(provenance) = &body.producer_provenance {
                        value["producer_provenance"] = provenance.as_json();
                    }
                    value
                })
            })
            .collect::<Vec<_>>();
        json!({
            "capacity": self.capacity,
            "delay": self.delay,
            "frame": self.frame,
            "write_slot": self.write_slot,
            "slots": slots,
            "buffer_ids": self.buffer_ids,
        })
    }

    fn operation_view(&self) -> Value {
        let slots = self
            .slots
            .iter()
            .map(|slot| {
                slot.as_ref().map_or(Value::Null, |body| {
                    Value::from(body.writer_generation.value())
                })
            })
            .collect::<Vec<_>>();
        json!({"slots": slots, "buffer_ids": self.buffer_ids})
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct QueueOperation {
    pub queue_id: u8,
    pub pushed_writer: CodecSourceIndex,
    pub pushed_bytes: u16,
    pub pushed_provenance: Option<MetadataQueueProducerProvenance>,
    pub consumer_transaction_id: Option<CallbackTransactionId>,
    pub consumer_source_index: Option<CodecSourceIndex>,
    pub consumer_dispatch: Option<JocDispatchIndex>,
    pub popped_writer: Option<CodecSourceIndex>,
    pub popped_buffer_id: u16,
    pub before: Value,
    pub after: Value,
    pub popped: Option<QueuedBody>,
}

impl QueueOperation {
    pub fn bind_consumer(
        &mut self,
        transaction_id: CallbackTransactionId,
        source_index: CodecSourceIndex,
        dispatch: JocDispatchIndex,
    ) {
        self.consumer_transaction_id = Some(transaction_id);
        self.consumer_source_index = Some(source_index);
        self.consumer_dispatch = Some(dispatch);
    }

    pub fn as_json(&self) -> Value {
        let mut value = json!({
            "queue_id": self.queue_id,
            "pushed_writer": self.pushed_writer.value(),
            "pushed_bytes": self.pushed_bytes,
            "popped_writer": self.popped_writer.map(CodecSourceIndex::value),
            "popped_buffer_id": self.popped_buffer_id,
            "before": self.before,
            "after": self.after,
        });
        if let Some(provenance) = &self.pushed_provenance {
            value["pushed_producer_provenance"] = provenance.as_json();
        }
        if let (Some(transaction_id), Some(source_index), Some(dispatch)) = (
            &self.consumer_transaction_id,
            self.consumer_source_index,
            self.consumer_dispatch,
        ) {
            value["consumer_transaction_id"] = Value::String(transaction_id.as_str().to_owned());
            value["consumer_source_index"] = Value::from(source_index.value());
            value["consumer_dispatch"] = Value::from(dispatch.value());
        }
        if let Some(provenance) = self
            .popped
            .as_ref()
            .and_then(|body| body.producer_provenance.as_ref())
        {
            value["popped_producer_provenance"] = provenance.as_json();
        }
        value
    }
}
