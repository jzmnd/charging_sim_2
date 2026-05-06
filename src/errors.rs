use thiserror::Error;

///
/// Simulation errors.
///
#[derive(Error, Debug)]
pub enum SimulationError {
    #[error("Invalid vehicle ID `{0}`")]
    InvalidVehicleId(String),
    #[error("Invalid charge profile ID `{0}`")]
    InvalidChargeProfileId(String),
    #[error("Invalid charge profile name `{0}`")]
    InvalidChargeProfileName(String),
    #[error("Invalid charger ID `{0}`")]
    InvalidChargerId(String),
    #[error("Missing charger ID from Event")]
    MissingChargerId,
    #[error("Charge profile should not be empty")]
    EmptyChargeProfile,
}

///
/// Object builder errors.
///
#[derive(Error, Debug)]
pub enum BuilderError {
    #[error("Missing charge profile ID")]
    MissingChargerProfileId,
    #[error(transparent)]
    SamplerError(#[from] SamplerError),
}

///
/// Sampler errors.
///
#[derive(Error, Debug)]
pub enum SamplerError {
    #[error("Invalid sampler parameter: {0}")]
    InvalidParameter(String),
}
