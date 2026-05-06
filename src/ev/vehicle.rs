use crate::distributions::{DurationSampler, ItemSampler, SocSampler};
use crate::errors::BuilderError;
use rand::Rng;
use uuid::Uuid;

#[derive(Debug)]
pub struct Vehicle {
    pub id: Uuid,
    pub name: String,
    pub soc_start: f64,
    pub soc_target: f64,
    pub charge_profile_id: Uuid,
    pub arrival_time: u64,
    pub idle_duration_s: f64,
    pub max_wait_s: u64,
    pub max_queue_length: usize,
}

impl Vehicle {
    ///
    /// Create a new vehicle builder.
    ///
    pub fn builder<'a>() -> VehicleBuilder<'a> {
        VehicleBuilder::default()
    }
}

const DEFAULT_AVG_SOC_START: f64 = 0.2;
const DEFAULT_AVG_SOC_TARGET: f64 = 0.8;
const DEFAULT_SOC_KAPPA_START: f64 = 8.0;
const DEFAULT_SOC_KAPPA_TARGET: f64 = 14.0;

const DEFAULT_AVG_IDLE_DURATION_S: f64 = 120.0;
const DEFAULT_IDLE_DURATION_SHAPE: f64 = 3.0;

const DEFAULT_MAX_WAIT_S: u64 = 1800;
const DEFAULT_MAX_QUEUE_LENGTH: usize = 10;

#[derive(Default)]
pub struct VehicleBuilder<'a> {
    soc_start: Option<f64>,
    soc_target: Option<f64>,
    avg_start_soc: Option<f64>,
    avg_target_soc: Option<f64>,
    soc_kappa_start: Option<f64>,
    soc_kappa_target: Option<f64>,
    charge_profile_id: Option<Uuid>,
    charge_profile_ids: Option<Vec<Uuid>>,
    charge_profile_weights: Option<Vec<f64>>,
    arrival_time: Option<u64>,
    idle_duration_s: Option<f64>,
    avg_idle_duration_s: Option<f64>,
    idle_duration_shape: Option<f64>,
    max_wait_s: Option<u64>,
    max_queue_length: Option<usize>,
    rng: Option<&'a mut dyn Rng>,
}

impl<'a> VehicleBuilder<'a> {
    ///
    /// Set the starting SOC for the vehicle.
    /// Do not set this if you want to sample random SOCs.
    ///
    pub fn soc_start(&mut self, val: f64) -> &mut Self {
        self.soc_start = Some(val);
        self
    }

    ///
    /// Set the target SOC for the vehicle.
    /// Do not set this if you want to sample random SOCs.
    ///
    pub fn soc_target(&mut self, val: f64) -> &mut Self {
        self.soc_target = Some(val);
        self
    }

    ///
    /// Set the average starting SOC for the vehicle to randomly sample a value.
    ///
    pub fn avg_start_soc(&mut self, val: f64) -> &mut Self {
        self.avg_start_soc = Some(val);
        self
    }

    ///
    /// Set the average target SOC for the vehicle to randomly sample a value.
    ///
    pub fn avg_target_soc(&mut self, val: f64) -> &mut Self {
        self.avg_target_soc = Some(val);
        self
    }

    ///
    /// Set the shape parameter of the starting SOC distribution for the vehicle.
    ///
    pub fn soc_kappa_start(&mut self, val: f64) -> &mut Self {
        self.soc_kappa_start = Some(val);
        self
    }

    ///
    /// Set the shape parameter of the target SOC distribution for the vehicle.
    ///
    pub fn soc_kappa_target(&mut self, val: f64) -> &mut Self {
        self.soc_kappa_target = Some(val);
        self
    }

    ///
    /// Set the unique charge profile ID for the vehicle.
    /// Do not set this if you want to sample random charge profiles.
    ///
    pub fn charge_profile_id(&mut self, val: Uuid) -> &mut Self {
        self.charge_profile_id = Some(val);
        self
    }

    ///
    /// Set the list of charge profile IDs to sample from.
    ///
    pub fn charge_profile_ids(&mut self, vals: &[Uuid]) -> &mut Self {
        self.charge_profile_ids = Some(vals.to_vec());
        self
    }

    ///
    /// Set the weights of charge profile IDs to sample from.
    ///
    pub fn charge_profile_weights(&mut self, vals: &[f64]) -> &mut Self {
        self.charge_profile_weights = Some(vals.to_vec());
        self
    }

    ///
    /// Set the arrival time of the vehicle.
    ///
    pub fn arrival_time(&mut self, val: u64) -> &mut Self {
        self.arrival_time = Some(val);
        self
    }

    ///
    /// Set the idle duration of the vehicle after charging.
    /// Do not set this if you want to sample random a random idle duration.
    ///
    pub fn idle_duration_s(&mut self, val: f64) -> &mut Self {
        self.idle_duration_s = Some(val);
        self
    }

    ///
    /// Set the average idle duration of the vehicle after charging to randomly sample a value.
    ///
    pub fn avg_idle_duration_s(&mut self, val: f64) -> &mut Self {
        self.avg_idle_duration_s = Some(val);
        self
    }

    ///
    /// Set the shape parameter of the idle duration distribution for the vehicle.
    ///
    pub fn idle_duration_shape(&mut self, val: f64) -> &mut Self {
        self.idle_duration_shape = Some(val);
        self
    }

    ///
    /// Set the maximum wait time for the vehicle before it leaves the queue.
    ///
    pub fn max_wait_s(&mut self, val: u64) -> &mut Self {
        self.max_wait_s = Some(val);
        self
    }

    ///
    /// Set the maximum waiting-queue length the vehicle will tolerate on arrival.
    /// If the queue is at or above this length when the vehicle arrives it baulks
    /// (leaves immediately without joining the queue).
    ///
    pub fn max_queue_length(&mut self, val: usize) -> &mut Self {
        self.max_queue_length = Some(val);
        self
    }

    ///
    /// Set the RNG used by `build` to sample any unset properties.
    /// If unset, `build` falls back to `rand::rng()`.
    ///
    pub fn rng<R: Rng + 'a>(&mut self, rng: &'a mut R) -> &mut Self {
        self.rng = Some(rng);
        self
    }

    ///
    /// Build a named vehicle.
    /// Will sample values for the vehicle properties if not set explicitly,
    /// drawing from the configured RNG (or `rand::rng()` if none was set).
    ///
    pub fn build(&mut self, name: &str) -> Result<Vehicle, BuilderError> {
        let mut rng_default;
        let rng = match self.rng.as_deref_mut() {
            Some(r) => r,
            None => {
                rng_default = rand::rng();
                &mut rng_default
            }
        };

        let soc_sampler = SocSampler::new(
            self.avg_start_soc.unwrap_or(DEFAULT_AVG_SOC_START),
            self.avg_target_soc.unwrap_or(DEFAULT_AVG_SOC_TARGET),
            self.soc_kappa_start.unwrap_or(DEFAULT_SOC_KAPPA_START),
            self.soc_kappa_target.unwrap_or(DEFAULT_SOC_KAPPA_TARGET),
        )?;
        let (soc_start_sampled, soc_target_sampled) = soc_sampler.sample(rng);

        let idle_duration_s = match self.idle_duration_s {
            Some(duration) => duration,
            None => {
                let idle_dur_sampler = DurationSampler::new(
                    self.avg_idle_duration_s
                        .unwrap_or(DEFAULT_AVG_IDLE_DURATION_S),
                    self.idle_duration_shape
                        .unwrap_or(DEFAULT_IDLE_DURATION_SHAPE),
                )?;
                idle_dur_sampler.sample(rng)
            }
        };
        let charge_profile_id = match self.charge_profile_id {
            Some(id) => id,
            None => {
                let mut id_sampler = ItemSampler::new(
                    self.charge_profile_ids
                        .as_deref()
                        .ok_or(BuilderError::MissingChargerProfileId)?,
                );
                if let Some(weights) = self.charge_profile_weights.as_deref() {
                    id_sampler.set_weights(weights);
                }
                id_sampler
                    .sample(rng)
                    .ok_or(BuilderError::MissingChargerProfileId)?
            }
        };

        Ok(Vehicle {
            id: Uuid::new_v4(),
            name: name.to_owned(),
            soc_start: self.soc_start.unwrap_or(soc_start_sampled),
            soc_target: self.soc_target.unwrap_or(soc_target_sampled),
            charge_profile_id,
            arrival_time: self.arrival_time.unwrap_or(0),
            idle_duration_s,
            max_wait_s: self.max_wait_s.unwrap_or(DEFAULT_MAX_WAIT_S),
            max_queue_length: self.max_queue_length.unwrap_or(DEFAULT_MAX_QUEUE_LENGTH),
        })
    }
}
