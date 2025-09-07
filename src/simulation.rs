use crate::charging_functions::{end_charging, start_charging};
use crate::containers::{ChargeProfileList, VehicleList};
use crate::events::EventType;
use crate::evse::Site;
use crate::session::Session;
use log::info;
use std::collections::VecDeque;
use uuid::Uuid;

///
/// Run the simulation given a site, list of vehicles and list of charge profiles.
///
pub fn run(
    mut site: Site,
    vehicle_list: VehicleList,
    charge_profile_list: ChargeProfileList,
) -> Vec<Session> {
    let mut event_queue = vehicle_list.generate_arrival_events();
    let mut waiting_queue: VecDeque<Uuid> = VecDeque::new();
    let mut sessions: Vec<Session> = Vec::new();

    while let Some(event) = event_queue.pop() {
        match event.event_type {
            EventType::Arrival => {
                info!("[t={}s] Vehicle {} arrives", event.time, event.vehicle_id);

                // Find an unoccupied charger otherwise add vehicle to the queue
                if let Some(charger) = site.get_unoccupied_charger_mut() {
                    let vehicle = vehicle_list.get_vehicle(&event.vehicle_id).unwrap();
                    let charge_profile = charge_profile_list
                        .get_charge_profile(&vehicle.charge_profile_id)
                        .unwrap();
                    start_charging(
                        event.time,
                        vehicle,
                        charger,
                        charge_profile,
                        &mut event_queue,
                        &mut sessions,
                    );
                } else {
                    info!(
                        "[t={}s] Vehicle {} added to queue",
                        event.time, event.vehicle_id
                    );
                    waiting_queue.push_back(event.vehicle_id);
                }
            }
            EventType::Unplug => {
                info!("[t={}s] Vehicle {} unplugged", event.time, event.vehicle_id);

                let charger = site.get_charger_mut(&event.charger_id.unwrap()).unwrap();
                end_charging(event.time, charger);

                // Start charging the next vehicle in the queue
                if let Some(next_vehicle_id) = waiting_queue.pop_front() {
                    let vehicle = vehicle_list.get_vehicle(&next_vehicle_id).unwrap();
                    let charge_profile = charge_profile_list
                        .get_charge_profile(&vehicle.charge_profile_id)
                        .unwrap();
                    start_charging(
                        event.time,
                        vehicle,
                        charger,
                        charge_profile,
                        &mut event_queue,
                        &mut sessions,
                    );
                }
            }
        }
    }
    sessions
}
