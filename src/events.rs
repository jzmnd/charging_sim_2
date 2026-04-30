use std::cmp::Ordering;
use uuid::Uuid;

///
/// Charging simulation event type.
///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    Arrival,
    Unplug,
}

///
/// Charging simulation event.
/// Event occurs at a given simulation time (in seconds).
///
#[derive(Debug, Eq)]
pub struct Event {
    pub(crate) time: u64,
    pub(crate) event_type: EventType,
    pub(crate) vehicle_id: Uuid,
    pub(crate) charger_id: Option<Uuid>,
}

impl Ord for Event {
    fn cmp(&self, other: &Self) -> Ordering {
        self.time.cmp(&other.time).reverse()
    }
}

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Event {
    fn eq(&self, other: &Self) -> bool {
        (self.time == other.time) && (self.event_type == other.event_type)
    }
}
