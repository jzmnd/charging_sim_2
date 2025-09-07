use std::cmp::Ordering;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    Arrival,
    Unplug,
}

#[derive(Debug, Eq)]
pub struct Event {
    pub time: u64,
    pub event_type: EventType,
    pub vehicle_id: Uuid,
    pub charger_id: Option<Uuid>,
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
