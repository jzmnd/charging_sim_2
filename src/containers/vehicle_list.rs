use crate::errors::SimulationError;
use crate::ev::Vehicle;
use crate::events::{Event, EventType};
use std::collections::BinaryHeap;
use std::collections::HashMap;
use uuid::Uuid;

///
/// A list of vehicles including a mapping of vehicle ID to vehicle objects.
///
#[derive(Debug, Default)]
pub struct VehicleList {
    vehicles: Vec<Vehicle>,
    vehicle_map: HashMap<Uuid, usize>,
}

impl VehicleList {
    ///
    /// Create a new list of vehicles.
    ///
    pub fn new(vehicles: Vec<Vehicle>) -> Self {
        let vehicle_map = vehicles
            .iter()
            .enumerate()
            .map(|(i, v)| (v.id, i))
            .collect();

        Self {
            vehicles,
            vehicle_map,
        }
    }

    ///
    /// Get the number of vehicles in the list.
    ///
    pub fn num_vehicles(&self) -> usize {
        self.vehicles.len()
    }

    ///
    /// Fetch a vehicle from the list using its ID.
    ///
    pub fn get_vehicle(&self, id: &Uuid) -> Result<&Vehicle, SimulationError> {
        self.vehicle_map
            .get(id)
            .map(|&idx| &self.vehicles[idx])
            .ok_or_else(|| SimulationError::InvalidVehicleId(id.to_string()))
    }

    ///
    /// Fetch a mutable vehicle from the list using its ID.
    ///
    pub fn get_vehicle_mut(&mut self, id: &Uuid) -> Result<&mut Vehicle, SimulationError> {
        self.vehicle_map
            .get(id)
            .map(|&idx| &mut self.vehicles[idx])
            .ok_or_else(|| SimulationError::InvalidVehicleId(id.to_string()))
    }

    ///
    /// Generate a list of arival events from the vehicle list.
    ///
    pub fn generate_arrival_events(&self) -> BinaryHeap<Event> {
        let mut events = BinaryHeap::new();
        for vehicle in &self.vehicles {
            events.push(Event {
                time: vehicle.arrival_time,
                event_type: EventType::Arrival,
                vehicle_id: vehicle.id,
                charger_id: None,
            });
            events.push(Event {
                time: vehicle.arrival_time + vehicle.max_wait_s,
                event_type: EventType::Renege,
                vehicle_id: vehicle.id,
                charger_id: None,
            });
        }
        events
    }
}

impl FromIterator<Vehicle> for VehicleList {
    fn from_iter<I: IntoIterator<Item = Vehicle>>(iter: I) -> Self {
        Self::new(iter.into_iter().collect())
    }
}

impl Extend<Vehicle> for VehicleList {
    fn extend<I: IntoIterator<Item = Vehicle>>(&mut self, iter: I) {
        for vehicle in iter {
            let idx = self.vehicles.len();
            self.vehicle_map.insert(vehicle.id, idx);
            self.vehicles.push(vehicle);
        }
    }
}
