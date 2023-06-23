//! Id Generator
pub(super) mod aws;

use opentelemetry_api::trace::{SpanId, TraceId};
use rand::{rngs, Rng};
use std::cell::RefCell;
use std::fmt;

/// Interface for generating IDs
pub trait IdGenerator: Send + Sync + fmt::Debug {
    /// Generate a new `TraceId`
    fn new_trace_id(&self, backward_compatible: Option<bool>) -> TraceId;

    /// Generate a new `SpanId`
    fn new_span_id(&self) -> SpanId;
}

/// Default [`IdGenerator`] implementation.
///
/// Generates Trace and Span ids using a random number generator.
#[derive(Clone, Debug, Default)]
pub struct RandomIdGenerator {
    _private: (),
}

impl IdGenerator for RandomIdGenerator {
    fn new_trace_id(&self, backward_compatible: Option<bool>) -> TraceId {   
        let backward_compatible_val: bool = backward_compatible == Some(true);
        if backward_compatible_val {
            let span_id_to_convert: SpanId = CURRENT_RNG.with(|rng| SpanId::from(rng.borrow_mut().gen::<u64>()));
            return TraceId::from_hex(&span_id_to_convert.to_string()).unwrap();
        }
        CURRENT_RNG.with(|rng| TraceId::from(rng.borrow_mut().gen::<u128>()))
    }

    fn new_span_id(&self) -> SpanId {
        CURRENT_RNG.with(|rng| SpanId::from(rng.borrow_mut().gen::<u64>()))
    }
}

thread_local! {
    /// Store random number generator for each thread
    static CURRENT_RNG: RefCell<rngs::ThreadRng> = RefCell::new(rngs::ThreadRng::default());
}
