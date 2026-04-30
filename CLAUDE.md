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
cargo test test_calculate               # one test by name
cargo test --package charging_sim_2 ev::charge_profile   # one module
```

Run the bundled examples. They read CSVs from `data/` and write to `outputs/`, so **always run from the project root**:

```bash
cargo run --example run_example_simulation
cargo run --example run_example_simulation_long   # ~1 year of synthetic arrivals
```

The simulation logs through the `log` crate; enable output with e.g. `RUST_LOG=info cargo run --example run_example_simulation` (`env_logger` is wired up in the lib but examples currently don't init it — add `env_logger::init()` to `main` if you need logs).

## Architecture

This is a **discrete-event simulation** of EV charging at a single site, structured as a library crate (`charging_sim_2`) with thin example binaries. The big picture is in `src/simulation.rs`; everything else is a building block fed into `Simulation::new(...)`.

### Event loop (the core)

`Simulation` owns four pieces of mutable state:

- `event_queue: BinaryHeap<Event>` — priority queue of pending events. `Event::Ord` is **reversed on `time`**, so the max-heap behaves as a min-heap by time. This is the only ordering mechanism — there is no global clock variable; "now" is whatever event you just popped.
- `waiting_queue: VecDeque<Uuid>` — FIFO of vehicle IDs that arrived while all chargers were busy.
- `site: Site` — chargers and their busy state.
- `sessions: Vec<Session>` — append-only output log; each charging start pushes one row.

`Simulation::run` pops events until the queue is empty. Two event types:

- `Arrival` → grab the first unoccupied charger (`Site::get_unoccupied_charger_mut`, linear scan), call `Charger::start_charging`. If none free, push the vehicle ID onto `waiting_queue`.
- `Unplug` → mark the charger free, then immediately pull the head of `waiting_queue` (if any) and start it on this charger at the current event time.

`Charger::start_charging` is where physics happens: it asks the `ChargeProfile` to integrate from `soc_start` to `soc_target` capped at the charger's effective max power (the lesser of `max_power_kw` and `max_current_a * voltage / 1000`), records a `Session`, and **pushes the matching `Unplug` event back onto the heap** (`now + charge_duration + idle_duration`). This self-scheduling is what makes the simulation progress without a fixed time step.

### Charge profile integration

`ChargeProfile` (`src/ev/charge_profile.rs`) is a sorted list of `(soc, power)` records plus a battery capacity. `calculate(soc_start, soc_target, max_power_kw)` walks the segments between adjacent records, clips each segment to the requested SOC range, and computes energy as `delta_soc * battery_capacity_kwh` and duration as `energy / avg_power` (with `power` linearly interpolated via `power_at` and capped at `max_power_kw`). This trapezoidal approximation is the source of all duration/energy/peak-power numbers in the output.

### Builders + sampling

`Vehicle` and `Charger` use builders. Setters are `&mut self -> &mut Self`, so a builder is reusable across many `build()` calls — configure once outside a loop, then vary one or two fields per iteration (see `run_example_simulation_long.rs`).

`VehicleBuilder` is non-trivial: any of `soc_start`, `soc_target`, `idle_duration_s`, and `charge_profile_id` can be left unset, in which case `build()` samples them:

- SOCs via beta distributions parameterized by mean and `kappa` (concentration); resampled until `target > start` (`distributions/soc_distr.rs`).
- Idle duration via gamma (`distributions/idle_distr.rs`).
- Charge profile via weighted or uniform pick from a list of IDs (`distributions/discrete_distr.rs`).

`charge_profile_id` is the one required field — `build()` returns `BuilderError::MissingChargerProfileId` if neither a single ID nor a list of IDs was provided.

#### RNG handling

`VehicleBuilder` is parameterised by a lifetime: `VehicleBuilder<'a>` holds an optional `&'a mut dyn Rng`. Configure it via `.rng(&mut my_rng)`; if unset, `build` falls back to a fresh `rand::rng()` (thread-local) per call. The sample functions in `src/distributions/*` all take an explicit `rng: &mut R` (or `&mut dyn Rng` for `ArrivalSampler::sample_arrivals`). For reproducibility, pass a seeded `StdRng::seed_from_u64(...)` instead — no library changes needed. Note: `rand` 0.10 dropped `RngCore` from its crate root; use `rand::Rng` for trait objects/bounds.

For bulk arrival generation, `ArrivalSampler` combines a 24-hour weight array, a 7-day weight array, and an `avg_sessions_per_day` Poisson rate to produce sorted arrival times in seconds. `run_example_simulation_long.rs` shows the canonical use: sample arrivals, then build one `Vehicle` per arrival from a single reusable builder.

### Containers

`VehicleList` and `ChargeProfileList` (in `src/containers/`) are `Vec` + `HashMap<Uuid, usize>` index pairs that give O(1) lookup by ID. They own their elements; the `Simulation` borrows from them by ID during the event loop. `VehicleList::generate_arrival_events` seeds the event queue at simulation start (called by `Simulation::new`).

Both containers also `impl FromIterator` and `impl Extend` for their element types, so you can `vehicles.into_iter().collect::<VehicleList>()` or grow a list incrementally with `list.extend(...)`.

Lookups (`get_vehicle`, `get_charge_profile`, `get_id_by_name`) return `Result<&T, SimulationError>` — the simulation just `?`s them, no manual `Option`-to-error mapping needed.

### Errors

All fallible operations return `Result<_, SimulationError>` or `Result<_, BuilderError>` (both `thiserror`-derived, in `src/errors.rs`). `ChargeProfile::calculate`, the container lookups, and `Site::get_charger_mut` are the main sources of `SimulationError`.

### Module layout convention

Each subdirectory under `src/` has a sibling `<dir>.rs` file that just `pub mod`s and re-exports the public types (e.g. `src/ev.rs` re-exports `ChargeProfile` and `Vehicle` from `src/ev/`). Add new modules by following the same pattern: create the file under the dir, then add `pub mod` + `pub use` to the sibling.

## Data and outputs

- `data/example_charge_profile_*.csv` — sample SOC-vs-power curves consumed by `ChargeProfile::from_file`. Format: `soc,power` header, ascending-ish SOC rows; the loader sorts on read.
- `outputs/` — destination for `Simulation::save_sessions_csv(path)`. Examples write here; the directory must exist.
