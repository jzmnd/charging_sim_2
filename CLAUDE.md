# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

Build / check / lint / format:

```bash
cargo build
cargo check
cargo clippy --all-targets
cargo fmt
```

Tests (unit tests live alongside the modules they cover, e.g. in `src/ev/charge_profile.rs`):

```bash
cargo test                              # all tests
cargo test test_integrate_over          # one test by name
cargo test --package charging_sim_2 ev::charge_profile   # one module
```

Run the bundled examples. They read CSVs from `data/` and write to `outputs/`, so **always run from the project root**:

```bash
cargo run --example run_example_simulation          # discrete-event sim
cargo run --example run_example_simulation_long     # discrete-event, ~1 year of synthetic arrivals
cargo run --example run_example_ts_simulation       # time-step sim
cargo run --example run_example_ts_simulation_long  # time-step, ~1 year of synthetic arrivals
```

Both engines log through the `log` crate; enable output with e.g. `RUST_LOG=info cargo run --example run_example_ts_simulation`.

## Architecture

This is a simulation of EV charging at a single site, structured as a library crate (`charging_sim_2`) with thin example binaries. There are **two simulation engines** sharing the same world (site, vehicles, charge profiles, sessions); pick one and feed it into `<Sim>::new(site, vehicle_list, charge_profile_list)`. Both live under `src/simulation/`:

- `DiscreteEventSimulation` (`src/simulation/discrete.rs`) — pure event-driven. Computes each charging session as a single self-contained integration at session start and schedules its matching `Unplug` event. Fast; cannot enforce time-varying constraints like site-wide power caps (it `log::warn`s at startup if `site_max_power_kw` is set).
- `TimeStepSimulation` (`src/simulation/timestep.rs`) — fixed 1-second timestep, integrates charging in-loop. Reads each vehicle's requested power from its `ChargeProfile` every tick, so power follows the SOC curve as it changes — and enforces site-wide power caps by derating active chargers.

### Discrete-event engine

`DiscreteEventSimulation` owns:

- `event_queue: BinaryHeap<Event>` — priority queue of pending events. `Event::Ord` is **reversed on `time`**, so the max-heap behaves as a min-heap by time. There is no clock variable; "now" is whatever event you just popped.
- `waiting_queue: VecDeque<Uuid>` — FIFO of vehicles that arrived while all chargers were busy.
- `site: Site` — chargers and their busy state.
- `sessions: Vec<Session>` — append-only output log.

`run` pops events until the queue is empty. Three event types:

- `Arrival` → grab the first unoccupied charger (`Site::get_unoccupied_charger_mut`, linear scan), call `Charger::start_charging_discrete`. If none free, push the vehicle ID onto `waiting_queue` (or balk if the queue is already at `max_queue_length`).
- `Unplug` → mark the charger free, then immediately pull the head of `waiting_queue` (if any) and start it on this charger at the current event time.
- `Renege` → remove the vehicle from the `waiting_queue` if present and record a reneged session. Renege events are pre-seeded at `arrival_time + max_wait_s` by `VehicleList::generate_arrival_events` and become no-ops once the vehicle has been served.

`Charger::start_charging_discrete` integrates the entire session up-front using `ChargeProfile::integrate_over(soc_start, soc_target, max_power_kw)`, where `max_power_kw` is `Charger::actual_max_power_kw()` (the lesser of `max_power_kw` and `max_current_a * voltage / 1000`). It records the `Session` and **pushes the matching `Unplug` event back onto the heap** at `now + charge_duration + idle_duration`. This self-scheduling is what makes the simulation progress without a fixed time step.

### Time-step engine

`TimeStepSimulation` owns the same world plus a `current_time: u64` clock, an `active_charger_states: FxHashMap<Uuid, ChargerState>`, and a `finished_chargers: Vec<Uuid>` reuse buffer. The main loop advances `current_time` by `TIMESTEP = 1` second per iteration and, each tick:

1. Drains all `Arrival`/`Renege` events whose `time <= current_time`. Arrivals either start charging immediately (creating a `ChargerState` via `Charger::start_charging_timestep`) or queue/balk.
2. Calls `Site::allocate_power(&mut active_charger_states, &charge_profile_list)` to set each active charger's `requested_power_kw` / `charger_max_power_kw` / `max_power_kw` for this tick (see below).
3. For each active charger: if `Charging`, deliver `min(requested_power_kw, max_power_kw)` for one timestep, update `current_soc`, flip to `Idle` once `current_soc >= target_soc`; if `Idle`, decrement `idle_remaining_s` and finalize the `Session` + free the charger when it reaches zero.
4. Removes finished chargers from `active_charger_states` and immediately pulls the next queued vehicle onto each freed one.

`ChargerState` (`src/evse/charger.rs`) caches everything the inner step needs — `battery_capacity_kwh`, `requested_power_kw`, `charger_max_power_kw`, `max_power_kw`, status, idle remaining, and the running session accumulators (`energy_kwh`, `peak_power_kw`, `session_idx`) — so the inner step itself does no `ChargeProfileList` or `Site` lookups.

### Site-wide power cap

`Site::site_max_power_kw: Option<f64>` is optional and only enforced by `TimeStepSimulation`. `Site::allocate_power` runs every tick and does two passes over `active_charger_states`:

1. For each active charger, compute its physical cap via `Charger::actual_max_power_kw()` and look up the vehicle's requested power via `ChargeProfile::power_at(current_soc)`. Cache both onto `ChargerState` (`requested_power_kw`, `charger_max_power_kw`). Accumulate `min(requested, physical_cap)` into a site total.
2. If `site_max_power_kw.is_some()` and `total_requested > site_max`, compute `scale = site_max / total_requested` and write `max_power_kw = min(requested, physical_cap) * scale` to every active charger. Otherwise `max_power_kw = physical_cap`.

The inner timestep loop then delivers `min(requested_power_kw, max_power_kw)`. When derated this works out to the proportionally-scaled value; when not, to the per-charger physical cap. `DiscreteEventSimulation` cannot enforce this — it computes each session's duration as a self-contained integration at session start, so a time-varying instantaneous constraint doesn't fit its execution model. It logs a warning and ignores the cap.

### Charge profile integration

`ChargeProfile` (`src/ev/charge_profile.rs`) is a sorted list of `(soc, power)` records plus a `battery_capacity_kwh`. Two entry points:

- `power_at(soc)` — linear interpolation between adjacent records. The hot path in `Site::allocate_power`.
- `integrate_over(soc_start, soc_target, max_power_kw)` — walks segments between records, clips each to the requested SOC range, and computes energy as `delta_soc * battery_capacity_kwh` and duration as `energy / avg_power` (with each segment's power linearly interpolated via `power_at` and capped at `max_power_kw`). Trapezoidal. This is what `start_charging_discrete` uses to compute a whole session in one shot.

The constructor extrapolates flat segments to SOC=0 and SOC=1 if the input data doesn't cover them.

### Builders + sampling

`Vehicle` and `Charger` use builders. Setters are `&mut self -> &mut Self`, so a builder is reusable across many `build()` calls — configure once outside a loop, then vary one or two fields per iteration (see the `*_long` examples).

`VehicleBuilder` is non-trivial: any of `soc_start`, `soc_target`, `idle_duration_s`, and `charge_profile_id` can be left unset, in which case `build()` samples them:

- SOCs via beta distributions parameterized by mean and `kappa` (concentration); resampled until `target > start` (`distributions/soc_distr.rs`).
- Idle duration via gamma distribution (`distributions/time_distr.rs`).
- Charge profile via weighted or uniform pick from a list of IDs (`distributions/discrete_distr.rs`).

`charge_profile_id` is the one required field — `build()` returns `BuilderError::MissingChargerProfileId` if neither a single ID nor a list of IDs was provided.

For bulk arrival generation, `ArrivalSampler` (`distributions/arrival_distr.rs`) combines a 24-hour weight array, a 7-day weight array, and an `avg_sessions_per_day` Poisson rate to produce sorted arrival times in seconds. The `*_long` examples show the canonical use: sample arrivals, then build one `Vehicle` per arrival from a single reusable builder.

#### RNG handling

`VehicleBuilder` is parameterised by a lifetime: `VehicleBuilder<'a>` holds an optional `&'a mut dyn Rng`. Configure it via `.rng(&mut my_rng)`; if unset, `build` falls back to a fresh `rand::rng()` (thread-local) per call. The sample functions in `src/distributions/*` all take an explicit `rng: &mut R` (or `&mut dyn Rng` for `ArrivalSampler::sample_arrivals`). For reproducibility, pass a seeded `StdRng::seed_from_u64(...)` instead — no library changes needed. Note: `rand` 0.10 dropped `RngCore` from its crate root; use `rand::Rng` for trait objects/bounds.

### Containers

`VehicleList` and `ChargeProfileList` (in `src/containers/`) are `Vec` + `FxHashMap<Uuid, usize>` index pairs that give O(1) lookup by ID. They own their elements; both simulations borrow from them by ID during the run loop. `VehicleList::generate_arrival_events` seeds the event queue (arrivals and the corresponding renege events) at simulation start — both `DiscreteEventSimulation::new` and `TimeStepSimulation::new` call it.

Both containers also `impl FromIterator` and `impl Extend` for their element types, so you can `vehicles.into_iter().collect::<VehicleList>()` or grow a list incrementally with `list.extend(...)`.

Lookups (`get_vehicle`, `get_charge_profile`, `get_id_by_name`) return `Result<&T, SimulationError>`.

### Errors

All fallible operations return `Result<_, SimulationError>` or `Result<_, BuilderError>` (both `thiserror`-derived, in `src/errors.rs`). `ChargeProfile::integrate_over` / `power_at`, the container lookups, and `Site::get_charger` / `get_charger_mut` are the main sources of `SimulationError`.

### Module layout convention

Each subdirectory under `src/` has a sibling `<dir>.rs` file that just `pub mod`s and re-exports the public types (e.g. `src/ev.rs` re-exports `ChargeProfile` and `Vehicle` from `src/ev/`; `src/simulation.rs` re-exports `DiscreteEventSimulation` and `TimeStepSimulation` from `src/simulation/`). Add new modules by following the same pattern: create the file under the dir, then add `pub mod` + `pub use` to the sibling.

## Data and outputs

- `data/example_charge_profile_*.csv` — sample SOC-vs-power curves consumed by `ChargeProfile::from_file`. Format: `soc,power` header, ascending-ish SOC rows; the loader sorts on read.
- `outputs/` — destination for `save_sessions_csv(path)` (available on both simulation types). Examples write here; the directory must exist.
