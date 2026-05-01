use crate::containers::{ChargeProfileList, VehicleList};
use crate::errors::SimulationError;
use crate::events::{Event, EventType};
use crate::evse::Site;
use crate::session::Session;
use log::info;
use std::collections::{BinaryHeap, VecDeque};
use std::path::Path;
use uuid::Uuid;

///
/// An EV charging simulation object.
///
#[derive(Debug)]
pub struct Simulation {
    site: Site,
    vehicle_list: VehicleList,
    charge_profile_list: ChargeProfileList,
    event_queue: BinaryHeap<Event>,
    waiting_queue: VecDeque<Uuid>,
    pub sessions: Vec<Session>,
}

impl Simulation {
    ///
    /// Create a new simulation object.
    ///
    pub fn new(
        site: Site,
        vehicle_list: VehicleList,
        charge_profile_list: ChargeProfileList,
    ) -> Self {
        let event_queue = vehicle_list.generate_arrival_events();
        Self {
            site,
            vehicle_list,
            charge_profile_list,
            event_queue,
            waiting_queue: VecDeque::new(),
            sessions: Vec::new(),
        }
    }

    ///
    /// Run the simulation given a site, list of vehicles and list of charge profiles.
    ///
    pub fn run(&mut self) -> Result<(), SimulationError> {
        while let Some(event) = self.event_queue.pop() {
            match event.event_type {
                EventType::Arrival => {
                    info!("[t={}s] Vehicle {} arrives", event.time, event.vehicle_id);

                    // Find an unoccupied charger otherwise add vehicle to the queue
                    if let Some(charger) = self.site.get_unoccupied_charger_mut() {
                        let vehicle = self.vehicle_list.get_vehicle(&event.vehicle_id)?;
                        let charge_profile = self
                            .charge_profile_list
                            .get_charge_profile(&vehicle.charge_profile_id)?;
                        charger.start_charging(
                            event.time,
                            vehicle,
                            charge_profile,
                            &mut self.event_queue,
                            &mut self.sessions,
                        )?;
                    } else {
                        info!(
                            "[t={}s] Vehicle {} added to queue",
                            event.time, event.vehicle_id
                        );
                        self.waiting_queue.push_back(event.vehicle_id);
                    }
                }
                EventType::Unplug => {
                    info!("[t={}s] Vehicle {} unplugged", event.time, event.vehicle_id);

                    let charger_id = event.charger_id.ok_or(SimulationError::MissingChargerId)?;
                    let charger = self.site.get_charger_mut(&charger_id)?;
                    charger.end_charging(event.time);

                    // Start charging the next vehicle in the queue
                    if let Some(next_vehicle_id) = self.waiting_queue.pop_front() {
                        let vehicle = self.vehicle_list.get_vehicle(&next_vehicle_id)?;
                        let charge_profile = self
                            .charge_profile_list
                            .get_charge_profile(&vehicle.charge_profile_id)?;
                        charger.start_charging(
                            event.time,
                            vehicle,
                            charge_profile,
                            &mut self.event_queue,
                            &mut self.sessions,
                        )?;
                    }
                }
            }
        }
        Ok(())
    }

    ///
    /// Save the simulation's sessions to a .csv file.
    ///
    pub fn save_sessions_csv<P: AsRef<Path>>(&self, path: P) -> Result<(), csv::Error> {
        let mut wtr = csv::Writer::from_path(path)?;
        for session in &self.sessions {
            wtr.serialize(session)?;
        }
        wtr.flush()?;
        Ok(())
    }
}
