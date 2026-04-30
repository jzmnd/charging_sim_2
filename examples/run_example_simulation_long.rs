use charging_sim_2::containers::{ChargeProfileList, VehicleList};
use charging_sim_2::distributions::ArrivalSampler;
use charging_sim_2::ev::{ChargeProfile, Vehicle};
use charging_sim_2::evse::{Charger, Site};
use charging_sim_2::simulation::Simulation;
use std::process;

fn main() {
    let charge_profile_list = ChargeProfileList::new(
        "cpl1",
        vec![
            ChargeProfile::from_file("cp1", "data/example_charge_profile_1.csv", 60.0).unwrap(),
            ChargeProfile::from_file("cp2", "data/example_charge_profile_2.csv", 75.0).unwrap(),
        ],
    );

    let mut vehicles: Vec<Vehicle> = vec![];

    let tod_distr = [
        0.0, 0.0, 0.05, 0.05, 0.1, 0.1, 0.15, 0.25, 0.3, 0.35, 0.4, 0.4, 0.35, 0.35, 0.4, 0.45,
        0.5, 0.55, 0.6, 0.5, 0.4, 0.3, 0.2, 0.1,
    ];
    let dow_distr = [0.95, 0.8, 0.8, 0.75, 0.8, 0.85, 1.0];
    let avg_sessions_per_day = 23.4;
    let arrival_sampler = ArrivalSampler::new(tod_distr, dow_distr, avg_sessions_per_day);

    for (i, &time) in arrival_sampler.sample_arrivals(365).iter().enumerate() {
        let v = Vehicle::builder()
            .charge_profile_ids(&charge_profile_list.all_ids())
            .charge_profile_weights(&[0.4, 0.6])
            .arrival_time(time)
            .build(&format!("v{:0>5}", i))
            .unwrap();

        vehicles.push(v);
    }

    let vehicle_list = VehicleList::new("vl1", vehicles);

    let site = Site::new(
        "s1",
        vec![
            Charger::builder()
                .max_power_kw(180.0)
                .max_current_a(500.0)
                .build("c1"),
            Charger::builder()
                .max_power_kw(180.0)
                .max_current_a(500.0)
                .build("c2"),
        ],
    );

    let mut simulation = Simulation::new(site, vehicle_list, charge_profile_list);
    if let Err(e) = simulation.run() {
        eprintln!("Simulation error: {}", e);
        process::exit(1);
    };

    for session in &simulation.sessions {
        println!("{:?}", session);
    }
    simulation
        .save_sessions_csv("outputs/example_simulation_long_output.csv")
        .unwrap();
}
