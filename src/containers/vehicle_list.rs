use crate::ev::Vehicle;
use crate::events::{Event, EventType};
use std::collections::BinaryHeap;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug)]
pub struct VehicleList {
    pub id: Uuid,
    pub name: String,
    pub vehicles: Vec<Vehicle>,
    vehicle_map: HashMap<Uuid, usize>,
}

impl VehicleList {
    ///
    /// Create a new list of vehicles.
    ///
    pub fn new(name: &str, vehicles: Vec<Vehicle>) -> Self {
        let vehicle_map = vehicles
            .iter()
            .enumerate()
            .map(|(i, v)| (v.id, i))
            .collect();

        Self {
            id: Uuid::new_v4(),
            name: name.to_owned(),
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
    pub fn get_vehicle(&self, id: &Uuid) -> Option<&Vehicle> {
        self.vehicle_map.get(id).map(|&idx| &self.vehicles[idx])
    }

    ///
    /// Fetch a mutable vehicle from the list using its ID.
    ///
    pub fn get_vehicle_mut(&mut self, id: &Uuid) -> Option<&mut Vehicle> {
        self.vehicle_map.get(id).map(|&idx| &mut self.vehicles[idx])
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
        }
        events
    }
}
