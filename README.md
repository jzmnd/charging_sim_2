# EV Charging Simulator

A simulation of electric vehicle (EV) charging.

Two simulation approaches are implemented:

- A discrete event-driven simulation that is fast but cannot account for charger power sharing or site wide power limits
- A full time-step simulation that is slower but allows for power sharing algorithms to be implemented

## Features

- Ability to simulate different EV station sizes and configuration
- Simulate different vehicle mixes with user provided charging profiles (power vs SOC)
- Specify each vehicle exactly or randomly sample vehicle parameters (SOC start and target, idle times, arrival times)
- Account for vehicle queuing including maximum wait times and maximum queue lengths
- Energy delivered and charging time are calculated by integrating the vehicle charge profile

## Outputs

The simulation outputs data corresponding to each vehicle including:

- Charging, waiting and idling durations
- Whether the vehicle charged or left the queue early
- Energy delivered during the charging session
- Session peak power

## Examples

Example simulations can be found in the `examples` directory and can be run with e.g.:

```bash
cargo run --example run_example_simulation
```

Logging can be enabled with e.g. `RUST_LOG=info`.

## Random sampling

When using the random samplers to generate vehicles and arrival times the following distributions are implemented.

**Arrival times**: Sampled using user provided hour-of-day and day-of-week distributions then uniform sampling for minutes and seconds.

**Charge profiles**: Weighted sampling from a list of user provided profiles.

**SOC start and target**: Sampled from two beta distribution with user provided mean and kappa.
Guaranteed to produce targets greater than starts.

**Idle duration**: Sampled from a gamma distribution with user provided mean and shape.
