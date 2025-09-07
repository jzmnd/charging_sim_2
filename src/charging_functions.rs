use crate::ev::{ChargeProfile, Vehicle};
use crate::events::{Event, EventType};
use crate::evse::Charger;
use crate::session::Session;
use log::debug;
use std::collections::BinaryHeap;

///
/// Start charging a vehicle.
///
pub fn start_charging(
    now: u64,
    vehicle: &Vehicle,
    charger: &mut Charger,
    charge_profile: &ChargeProfile,
    event_queue: &mut BinaryHeap<Event>,
    sessions: &mut Vec<Session>,
) {
    let wait_duration_s = now - vehicle.arrival_time;
    let charge_outputs =
        charge_profile.calculate(vehicle.soc_start, vehicle.soc_target, charger.max_power_kw);

    let unplug_time =
        now + charge_outputs.duration_s.ceil() as u64 + vehicle.idle_duration_s.ceil() as u64;

    charger.is_busy = true;
    charger.occupied_until = unplug_time;

    debug!(
        "[t={}s] Vehicle {} starts charging on Charger {} until t={}s",
        now, vehicle.id, charger.id, unplug_time
    );

    sessions.push(Session {
        vehicle: vehicle.name.to_owned(),
        vehicle_id: vehicle.id,
        charge_profile: charge_profile.name.to_owned(),
        charge_profile_id: vehicle.charge_profile_id,
        charger: charger.name.to_owned(),
        charger_id: charger.id,
        arrival_time: vehicle.arrival_time,
        plugin_time: now,
        unplug_time,
        wait_duration_s,
        charge_duration_s: charge_outputs.duration_s,
        idle_duration_s: vehicle.idle_duration_s,
        max_power_kw: charge_outputs.peak_power_kw,
        energy_kwh: charge_outputs.energy_kwh,
        start_soc: vehicle.soc_start,
        end_soc: vehicle.soc_target,
    });

    event_queue.push(Event {
        time: unplug_time,
        event_type: EventType::Unplug,
        vehicle_id: vehicle.id,
        charger_id: Some(charger.id),
    });
}

///
/// Stop charging a vehicle.
///
pub fn end_charging(_now: u64, charger: &mut Charger) {
    charger.is_busy = false;
}
