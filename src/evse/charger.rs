use crate::errors::SimulationError;
use crate::ev::{ChargeProfile, Vehicle};
use crate::events::{Event, EventType};
use crate::session::Session;
use log::debug;
use std::collections::BinaryHeap;
use uuid::Uuid;

///
/// The status of an EV charger when a vehicle is connected.
///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChargerStatus {
    Charging,
    Idle,
}

///
/// Object that represents the state of an EV charger when a vehicle is connected.
///
#[derive(Debug)]
pub struct ChargerState {
    pub charge_profile_id: Uuid,
    pub current_soc: f64,
    pub current_max_power_kw: f64,
    pub target_soc: f64,
    pub idle_remaining_s: f64,
    pub status: ChargerStatus,
    pub energy_kwh: f64,
    pub peak_power_kw: f64,
    pub session_idx: usize,
}

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
    /// Marks the charger busy, schedules the matching `Unplug` event and
    /// records a charging `Session`.
    ///
    pub fn start_charging_discrete(
        &mut self,
        now: u64,
        vehicle: &Vehicle,
        charge_profile: &ChargeProfile,
        event_queue: &mut BinaryHeap<Event>,
        sessions: &mut Vec<Session>,
    ) -> Result<(), SimulationError> {
        let max_power_kw = (self.max_current_a * self.voltage / 1000.0).min(self.max_power_kw);
        let charge_outputs =
            charge_profile.integrate_over(vehicle.soc_start, vehicle.soc_target, max_power_kw)?;
        let unplug_time =
            now + charge_outputs.duration_s.ceil() as u64 + vehicle.idle_duration_s.ceil() as u64;

        self.is_busy = true;
        debug!(
            "[t={}s] Vehicle {} starts charging on Charger {} until t={}s",
            now, vehicle.id, self.id, unplug_time
        );

        sessions.push(Session::charged(
            now,
            unplug_time,
            vehicle,
            self,
            charge_profile,
            &charge_outputs,
        ));

        event_queue.push(Event {
            time: unplug_time,
            event_type: EventType::Unplug,
            vehicle_id: vehicle.id,
            charger_id: Some(self.id),
        });

        Ok(())
    }

    ///
    /// Start charging a vehicle.
    /// Marks the charger busy, starts a charging `Session`, and returns
    /// the charger state.
    ///
    pub fn start_charging_timestep(
        &mut self,
        now: u64,
        vehicle: &Vehicle,
        charge_profile: &ChargeProfile,
        sessions: &mut Vec<Session>,
    ) -> ChargerState {
        self.is_busy = true;
        debug!(
            "[t={}s] Vehicle {} starts charging on Charger {}",
            now, vehicle.id, self.id
        );

        sessions.push(Session::started(now, vehicle, self, charge_profile));
        let session_idx = sessions.len() - 1;

        ChargerState {
            charge_profile_id: charge_profile.id,
            current_soc: vehicle.soc_start,
            current_max_power_kw: self.max_power_kw,
            target_soc: vehicle.soc_target,
            idle_remaining_s: vehicle.idle_duration_s,
            status: ChargerStatus::Charging,
            energy_kwh: 0.0,
            peak_power_kw: 0.0,
            session_idx,
        }
    }

    ///
    /// Stop charging a vehicle.
    /// Marks the charger free to use.
    ///
    pub fn end_charging(&mut self, now: u64) {
        self.is_busy = false;
        debug!("[t={}s] Charging complete on Charger {}", now, self.id);
    }
}

const DEFAULT_MAX_POWER_KW: f64 = 480.0;
const DEFAULT_MAX_CURRENT_A: f64 = 1200.0;
const DEFAULT_VOLTAGE: f64 = 400.0;

///
/// Builder used to create `Charger` objects.
///
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
            is_busy: false,
        }
    }
}
