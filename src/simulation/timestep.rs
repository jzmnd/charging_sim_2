use crate::containers::{ChargeProfileList, VehicleList};
use crate::errors::SimulationError;
use crate::events::Event;
use crate::events::EventType;
use crate::evse::{ChargerState, ChargerStatus, Site};
use crate::session::Session;
use log::info;
use rustc_hash::FxHashMap;
use std::collections::{BinaryHeap, VecDeque};
use std::path::Path;
use uuid::Uuid;

const TIMESTEP: u64 = 1;
const TIMESTEP_F: f64 = TIMESTEP as f64;
const TIMESTEP_HRS_F: f64 = TIMESTEP_F / 3600.0;

///
/// A time-step EV charging simulation object.
///
#[derive(Debug)]
pub struct TimeStepSimulation {
    site: Site,
    vehicle_list: VehicleList,
    charge_profile_list: ChargeProfileList,
    event_queue: BinaryHeap<Event>,
    waiting_queue: VecDeque<Uuid>,
    sessions: Vec<Session>,
    current_time: u64,
    active_charger_states: FxHashMap<Uuid, ChargerState>,
    finished_chargers: Vec<Uuid>,
}

impl TimeStepSimulation {
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
            current_time: 0,
            active_charger_states: FxHashMap::default(),
            finished_chargers: Vec::new(),
        }
    }

    ///
    /// Run the simulation given a site, list of vehicles and list of charge profiles.
    ///
    pub fn run(&mut self) -> Result<(), SimulationError> {
        while !self.event_queue.is_empty()
            || !self.active_charger_states.is_empty()
            || !self.waiting_queue.is_empty()
        {
            // Process all arrival/renege events due at or before this time
            while let Some(event) = self.event_queue.peek()
                && event.time <= self.current_time
            {
                let event = self.event_queue.pop().expect("Already peeked");
                match event.event_type {
                    EventType::Arrival => {
                        info!("[t={}s] Vehicle {} arrives", event.time, event.vehicle_id);

                        let vehicle = self.vehicle_list.get_vehicle(&event.vehicle_id)?;
                        let charge_profile = self
                            .charge_profile_list
                            .get_charge_profile(&vehicle.charge_profile_id)?;

                        if let Some(charger) = self.site.get_unoccupied_charger_mut() {
                            let charger_id = charger.id;
                            let state = charger.start_charging_timestep(
                                event.time,
                                vehicle,
                                charge_profile,
                                &mut self.sessions,
                            );
                            self.active_charger_states.insert(charger_id, state);
                        } else if self.waiting_queue.len() >= vehicle.max_queue_length {
                            info!(
                                "[t={}s] Vehicle {} baulks (queue length {})",
                                event.time,
                                event.vehicle_id,
                                self.waiting_queue.len()
                            );
                            self.sessions.push(Session::balked(vehicle, charge_profile));
                        } else {
                            info!(
                                "[t={}s] Vehicle {} added to queue",
                                event.time, event.vehicle_id
                            );
                            self.waiting_queue.push_back(event.vehicle_id);
                        }
                    }
                    EventType::Renege => {
                        if let Some(pos) = self
                            .waiting_queue
                            .iter()
                            .position(|id| id == &event.vehicle_id)
                        {
                            self.waiting_queue.remove(pos);
                            let vehicle = self.vehicle_list.get_vehicle(&event.vehicle_id)?;
                            let charge_profile = self
                                .charge_profile_list
                                .get_charge_profile(&vehicle.charge_profile_id)?;
                            self.sessions.push(Session::reneged(
                                event.time,
                                vehicle,
                                charge_profile,
                            ));
                            info!(
                                "[t={}s] Vehicle {} reneges queue",
                                event.time, event.vehicle_id
                            );
                        }
                    }
                    EventType::Unplug => {
                        unreachable!("Unplug events are not used in the timestep simulation")
                    }
                }
            }

            // Compute the per-charger power cap and increment charging at each active charger
            self.site.allocate_power(&mut self.active_charger_states)?;

            for (charger_id, charger_state) in self.active_charger_states.iter_mut() {
                match charger_state.status {
                    ChargerStatus::Charging => {
                        let charge_profile = self
                            .charge_profile_list
                            .get_charge_profile(&charger_state.charge_profile_id)?;
                        let requested_power = charge_profile.power_at(charger_state.current_soc)?;
                        let delivered_power =
                            requested_power.min(charger_state.current_max_power_kw);
                        let delta_energy = delivered_power * TIMESTEP_HRS_F;

                        charger_state.energy_kwh += delta_energy;
                        charger_state.peak_power_kw =
                            charger_state.peak_power_kw.max(delivered_power);
                        charger_state.current_soc +=
                            delta_energy / charge_profile.battery_capacity_kwh;

                        if charger_state.current_soc >= charger_state.target_soc {
                            charger_state.status = ChargerStatus::Idle;
                        }
                    }
                    ChargerStatus::Idle => {
                        charger_state.idle_remaining_s -= TIMESTEP_F;
                        if charger_state.idle_remaining_s <= 0.0 {
                            self.sessions[charger_state.session_idx].finalize(
                                self.current_time,
                                charger_state.peak_power_kw,
                                charger_state.energy_kwh,
                            )?;
                            self.site
                                .get_charger_mut(charger_id)?
                                .end_charging(self.current_time);
                            self.finished_chargers.push(*charger_id);
                        }
                    }
                }
            }

            // Free finished chargers and immediately start the next queued vehicle
            for charger_id in self.finished_chargers.drain(..) {
                self.active_charger_states.remove(&charger_id);
                if let Some(next_vehicle_id) = self.waiting_queue.pop_front() {
                    let vehicle = self.vehicle_list.get_vehicle(&next_vehicle_id)?;
                    let charge_profile = self
                        .charge_profile_list
                        .get_charge_profile(&vehicle.charge_profile_id)?;
                    let charger = self.site.get_charger_mut(&charger_id)?;
                    let state = charger.start_charging_timestep(
                        self.current_time,
                        vehicle,
                        charge_profile,
                        &mut self.sessions,
                    );
                    self.active_charger_states.insert(charger_id, state);
                }
            }

            self.current_time += TIMESTEP;
        }
        Ok(())
    }

    ///
    /// Get the simulation's session data.
    ///
    pub fn get_sessions(&self) -> &[Session] {
        &self.sessions
    }

    ///
    /// Save the simulation's session data to a .csv file.
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
