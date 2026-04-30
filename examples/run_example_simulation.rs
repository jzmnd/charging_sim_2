use charging_sim_2::containers::{ChargeProfileList, VehicleList};
use charging_sim_2::ev::{ChargeProfile, Vehicle};
use charging_sim_2::evse::{Charger, Site};
use charging_sim_2::simulation::Simulation;
use std::process;

fn main() {
    let charge_profiles = vec![
        ChargeProfile::from_file("cp1", "data/example_charge_profile_1.csv", 60.0).unwrap(),
        ChargeProfile::from_file("cp2", "data/example_charge_profile_2.csv", 75.0).unwrap(),
    ];
    let charge_profile_list = ChargeProfileList::new("cpl1", charge_profiles);
    let cp1_id = charge_profile_list.get_id_by_name("cp1").unwrap();
    let cp2_id = charge_profile_list.get_id_by_name("cp2").unwrap();
    let all_ids = charge_profile_list.all_ids();

    let vehicles: Vec<Vehicle> = vec![
        Vehicle::builder()
            .soc_start(0.2)
            .soc_target(0.8)
            .charge_profile_id(cp1_id)
            .arrival_time(5 * 60)
            .build("v1")
            .unwrap(),
        Vehicle::builder()
            .soc_start(0.1)
            .soc_target(0.9)
            .charge_profile_ids(&all_ids)
            .arrival_time(30 * 60)
            .idle_duration_s(1.0 * 60.0)
            .build("v2")
            .unwrap(),
        Vehicle::builder()
            .soc_start(0.15)
            .soc_target(0.5)
            .charge_profile_ids(&all_ids)
            .arrival_time(30 * 60)
            .idle_duration_s(3.0 * 60.0)
            .build("v3")
            .unwrap(),
        Vehicle::builder()
            .soc_start(0.5)
            .soc_target(0.95)
            .charge_profile_id(cp1_id)
            .arrival_time(35 * 60)
            .build("v4")
            .unwrap(),
        Vehicle::builder()
            .soc_start(0.25)
            .soc_target(0.65)
            .charge_profile_id(cp2_id)
            .arrival_time(50 * 60)
            .idle_duration_s(1.0 * 60.0)
            .build("v5")
            .unwrap(),
        Vehicle::builder()
            .charge_profile_id(cp1_id)
            .arrival_time(52 * 60)
            .idle_duration_s(2.0 * 60.0)
            .build("v6")
            .unwrap(),
    ];
    let vehicle_list = VehicleList::new("vl1", vehicles);

    let chargers: Vec<Charger> = vec![
        Charger::builder()
            .max_power_kw(180.0)
            .max_current_a(100.0)
            .build("c1"),
        Charger::builder()
            .max_power_kw(180.0)
            .max_current_a(100.0)
            .build("c2"),
    ];
    let site = Site::new("s1", chargers);

    let mut simulation = Simulation::new(site, vehicle_list, charge_profile_list);
    if let Err(e) = simulation.run() {
        eprintln!("Simulation error: {}", e);
        process::exit(1);
    };

    for session in &simulation.sessions {
        println!("{:?}", session);
    }
    simulation
        .save_sessions_csv("outputs/example_simulation_output.csv")
        .unwrap();
}
