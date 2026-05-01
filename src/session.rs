use crate::ev::{ChargeProfile, ChargingOutput, Vehicle};
use crate::evse::Charger;
use serde::Serialize;
use uuid::Uuid;

///
/// Charging session data.
/// Reneged sessions (vehicle left the waiting queue before being charged) and
/// balked sessions (vehicle never joined the queue because it was too long)
/// have the charger and charging data fields as `None`.
///
#[derive(Debug, Serialize)]
pub struct Session {
    pub vehicle: String,
    pub vehicle_id: Uuid,
    pub charge_profile: String,
    pub charge_profile_id: Uuid,
    pub charger: Option<String>,
    pub charger_id: Option<Uuid>,
    pub arrival_time: u64,
    pub plugin_time: Option<u64>,
    pub unplug_time: Option<u64>,
    pub wait_duration_s: u64,
    pub reneged: bool,
    pub balked: bool,
    pub charge_duration_s: Option<f64>,
    pub idle_duration_s: Option<f64>,
    pub peak_power_kw: Option<f64>,
    pub energy_kwh: Option<f64>,
    pub start_soc: f64,
    pub end_soc: Option<f64>,
}

impl Session {
    ///
    /// Build a session for a vehicle that successfully started charging.
    ///
    pub fn charging(
        now: u64,
        unplug_time: u64,
        vehicle: &Vehicle,
        charger: &Charger,
        charge_profile: &ChargeProfile,
        charge_outputs: &ChargingOutput,
    ) -> Self {
        Self {
            vehicle: vehicle.name.to_owned(),
            vehicle_id: vehicle.id,
            charge_profile: charge_profile.name.to_owned(),
            charge_profile_id: vehicle.charge_profile_id,
            charger: Some(charger.name.to_owned()),
            charger_id: Some(charger.id),
            arrival_time: vehicle.arrival_time,
            plugin_time: Some(now),
            unplug_time: Some(unplug_time),
            wait_duration_s: now - vehicle.arrival_time,
            reneged: false,
            balked: false,
            charge_duration_s: Some(charge_outputs.duration_s),
            idle_duration_s: Some(vehicle.idle_duration_s),
            peak_power_kw: Some(charge_outputs.peak_power_kw),
            energy_kwh: Some(charge_outputs.energy_kwh),
            start_soc: vehicle.soc_start,
            end_soc: Some(vehicle.soc_target),
        }
    }

    ///
    /// Build a session for a vehicle that left the waiting queue before being charged.
    ///
    pub fn reneged(now: u64, vehicle: &Vehicle, charge_profile: &ChargeProfile) -> Self {
        Self {
            vehicle: vehicle.name.to_owned(),
            vehicle_id: vehicle.id,
            charge_profile: charge_profile.name.to_owned(),
            charge_profile_id: vehicle.charge_profile_id,
            charger: None,
            charger_id: None,
            arrival_time: vehicle.arrival_time,
            plugin_time: None,
            unplug_time: None,
            wait_duration_s: now - vehicle.arrival_time,
            reneged: true,
            balked: false,
            charge_duration_s: None,
            idle_duration_s: None,
            peak_power_kw: None,
            energy_kwh: None,
            start_soc: vehicle.soc_start,
            end_soc: None,
        }
    }

    ///
    /// Build a session for a vehicle that arrived but never joined the queue
    /// because it was already at or above the vehicle's tolerated queue length.
    ///
    pub fn balked(vehicle: &Vehicle, charge_profile: &ChargeProfile) -> Self {
        Self {
            vehicle: vehicle.name.to_owned(),
            vehicle_id: vehicle.id,
            charge_profile: charge_profile.name.to_owned(),
            charge_profile_id: vehicle.charge_profile_id,
            charger: None,
            charger_id: None,
            arrival_time: vehicle.arrival_time,
            plugin_time: None,
            unplug_time: None,
            wait_duration_s: 0,
            reneged: false,
            balked: true,
            charge_duration_s: None,
            idle_duration_s: None,
            peak_power_kw: None,
            energy_kwh: None,
            start_soc: vehicle.soc_start,
            end_soc: None,
        }
    }
}
