use crate::errors::SimulationError;
use crate::ev::{ChargeProfile, Vehicle};
use crate::events::{Event, EventType};
use crate::session::Session;
use log::debug;
use std::collections::BinaryHeap;
use uuid::Uuid;

///
/// Object that represents an EV charger.
///
#[derive(Debug)]
pub struct Charger {
    pub id: Uuid,
    pub name: String,
    pub max_power_kw: f64,
    pub max_current_a: f64,
    pub voltage: f64,
    pub occupied_until: u64,
    pub is_busy: bool,
}

impl Charger {
    ///
    /// Create a new EV charger builder.
    ///
    pub fn builder() -> ChargerBuilder {
        ChargerBuilder::default()
    }

    ///
    /// Start charging a vehicle.
    ///
    pub fn start_charging(
        &mut self,
        now: u64,
        vehicle: &Vehicle,
        charge_profile: &ChargeProfile,
        event_queue: &mut BinaryHeap<Event>,
        sessions: &mut Vec<Session>,
    ) -> Result<(), SimulationError> {
        let wait_duration_s = now - vehicle.arrival_time;
        let max_power_kw = (self.max_current_a * self.voltage / 1000.0).min(self.max_power_kw);
        let charge_outputs =
            charge_profile.calculate(vehicle.soc_start, vehicle.soc_target, max_power_kw)?;

        let unplug_time =
            now + charge_outputs.duration_s.ceil() as u64 + vehicle.idle_duration_s.ceil() as u64;

        self.is_busy = true;
        self.occupied_until = unplug_time;

        debug!(
            "[t={}s] Vehicle {} starts charging on Charger {} until t={}s",
            now, vehicle.id, self.id, unplug_time
        );

        sessions.push(Session {
            vehicle: vehicle.name.to_owned(),
            vehicle_id: vehicle.id,
            charge_profile: charge_profile.name.to_owned(),
            charge_profile_id: vehicle.charge_profile_id,
            charger: Some(self.name.to_owned()),
            charger_id: Some(self.id),
            arrival_time: vehicle.arrival_time,
            plugin_time: Some(now),
            unplug_time: Some(unplug_time),
            wait_duration_s,
            reneged: false,
            balked: false,
            charge_duration_s: Some(charge_outputs.duration_s),
            idle_duration_s: Some(vehicle.idle_duration_s),
            peak_power_kw: Some(charge_outputs.peak_power_kw),
            energy_kwh: Some(charge_outputs.energy_kwh),
            start_soc: vehicle.soc_start,
            end_soc: Some(vehicle.soc_target),
        });

        event_queue.push(Event {
            time: unplug_time,
            event_type: EventType::Unplug,
            vehicle_id: vehicle.id,
            charger_id: Some(self.id),
        });

        Ok(())
    }

    ///
    /// Stop charging a vehicle.
    ///
    pub fn end_charging(&mut self, now: u64) {
        self.is_busy = false;

        debug!("[t={}s] Charging complete on Charger {}", now, self.id);
    }
}

const DEFAULT_MAX_POWER_KW: f64 = 480.0;
const DEFAULT_MAX_CURRENT_A: f64 = 1200.0;
const DEFAULT_VOLTAGE: f64 = 400.0;

#[derive(Debug, Default)]
pub struct ChargerBuilder {
    max_power_kw: Option<f64>,
    max_current_a: Option<f64>,
    voltage: Option<f64>,
}

impl ChargerBuilder {
    ///
    /// Set the maximum power (kW) of the charger.
    ///
    pub fn max_power_kw(&mut self, val: f64) -> &mut Self {
        self.max_power_kw = Some(val);
        self
    }

    ///
    /// Set the maximum deliverable current (A) of the charger.
    ///
    pub fn max_current_a(&mut self, val: f64) -> &mut Self {
        self.max_current_a = Some(val);
        self
    }

    ///
    /// Set the voltage (V) of the charger.
    ///
    pub fn voltage(&mut self, val: f64) -> &mut Self {
        self.voltage = Some(val);
        self
    }

    ///
    /// Build a named charger.
    ///
    pub fn build(&self, name: &str) -> Charger {
        Charger {
            id: Uuid::new_v4(),
            name: name.to_owned(),
            max_power_kw: self.max_power_kw.unwrap_or(DEFAULT_MAX_POWER_KW),
            max_current_a: self.max_current_a.unwrap_or(DEFAULT_MAX_CURRENT_A),
            voltage: self.voltage.unwrap_or(DEFAULT_VOLTAGE),
            occupied_until: 0,
            is_busy: false,
        }
    }
}
