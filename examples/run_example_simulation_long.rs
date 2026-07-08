use charging_sim_2::containers::{ChargeProfileList, VehicleList};
use charging_sim_2::distributions::ArrivalSampler;
use charging_sim_2::ev::{ChargeProfile, Vehicle};
use charging_sim_2::evse::{Charger, Site};
use charging_sim_2::geo::Coords;
use charging_sim_2::simulation::DiscreteEventSimulation;
use std::process;

fn main() {
    env_logger::init();

    // Create a random number generator for sampling
    let mut rng = rand::rng();

    // Set up all the vehicle charge profiles required in the simulation
    let charge_profile_list = ChargeProfileList::new(vec![
        ChargeProfile::from_file("cp1", "data/example_charge_profile_1.csv", 60.0).unwrap(),
        ChargeProfile::from_file("cp2", "data/example_charge_profile_2.csv", 75.0).unwrap(),
    ]);

    // Randomly sample vehicle arrival times based on a time of day and
    // day of week distribution
    let tod_distr = [
        0.0, 0.0, 0.05, 0.05, 0.1, 0.1, 0.15, 0.25, 0.3, 0.35, 0.4, 0.4, 0.35, 0.35, 0.4, 0.45,
        0.5, 0.55, 0.6, 0.5, 0.4, 0.3, 0.2, 0.1,
    ];
    let dow_distr = [0.95, 0.8, 0.8, 0.75, 0.8, 0.85, 1.0];
    let avg_sessions_per_day = 23.4;
    let arrival_sampler = ArrivalSampler::new(tod_distr, dow_distr, avg_sessions_per_day).unwrap();
    let arrivals = arrival_sampler.sample(365, &mut rng);

    // Build the vehicle list by sampling vehicles for each arrival time
    let mut vehicle_builder = Vehicle::builder();
    vehicle_builder
        .charge_profile_ids(&charge_profile_list.all_ids())
        .charge_profile_weights(&[0.4, 0.6])
        .rng(&mut rng);

    let vehicle_list: VehicleList = arrivals
        .iter()
        .enumerate()
        .map(|(i, &time)| {
            vehicle_builder
                .arrival_time(time)
                .build(&format!("v{:0>5}", i))
                .unwrap()
        })
        .collect();

    // Build the EV chargers and site
    let mut charger_builder = Charger::builder();
    charger_builder.max_power_kw(180.0).max_current_a(500.0);

    let chargers: Vec<Charger> = (1..3)
        .map(|i| charger_builder.build(&format!("c{}", i)))
        .collect();
    let site = Site::new("s1", chargers, Coords::try_new(37.7893, -122.4014).unwrap());

    // Run the simulation and save outputs
    let mut simulation = DiscreteEventSimulation::new(site, vehicle_list, charge_profile_list);
    if let Err(e) = simulation.run() {
        eprintln!("Simulation error: {}", e);
        process::exit(1);
    };

    simulation
        .save_sessions_csv("outputs/example_simulation_long_output.csv")
        .unwrap();
}
