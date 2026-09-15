use crate::containers::VehicleList;
use crate::evse::Charger;
use std::collections::VecDeque;
use uuid::Uuid;

///
/// FIFO queue of vehicle IDs waiting for a compatible charger to free up.
///
#[derive(Debug, Default)]
pub struct WaitingQueue(VecDeque<Uuid>);

impl WaitingQueue {
    ///
    /// Create a new, empty waiting queue.
    ///
    pub fn new() -> Self {
        Self(VecDeque::new())
    }

    ///
    /// Get the number of vehicles currently waiting.
    ///
    pub fn len(&self) -> usize {
        self.0.len()
    }

    ///
    /// Check whether the queue is empty.
    ///
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    ///
    /// Add a vehicle to the back of the queue.
    ///
    pub fn push_back(&mut self, id: Uuid) {
        self.0.push_back(id);
    }

    ///
    /// Remove a vehicle from the queue by ID, wherever it is in the queue.
    /// Returns `true` if the vehicle was queued and has been removed.
    ///
    pub fn remove(&mut self, id: &Uuid) -> bool {
        match self.0.iter().position(|queued_id| queued_id == id) {
            Some(pos) => {
                self.0.remove(pos);
                true
            }
            None => false,
        }
    }

    ///
    /// Remove and return the first queued vehicle whose connectors are
    /// compatible with `charger`. Returns `None` if no queued vehicle matches.
    ///
    pub fn pop_compatible(&mut self, vehicles: &VehicleList, charger: &Charger) -> Option<Uuid> {
        let pos = self.0.iter().position(|id| {
            vehicles
                .get_vehicle(id)
                .map(|v| charger.resolve_connector(&v.connectors).is_some())
                .unwrap_or(false)
        })?;
        self.0.remove(pos)
    }
}
