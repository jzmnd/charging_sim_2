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
    #[error("Invalid site ID `{0}`")]
    InvalidSiteId(String),
    #[error("Invalid location ID `{0}`")]
    InvalidLocationId(String),
    #[error("Missing charger ID from Event")]
    MissingChargerId,
    #[error("Charge profile should not be empty")]
    EmptyChargeProfile,
    #[error("Plugin time is missing from the session data")]
    MissingSessionPluginTime,
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

///
/// Geolocation errors.
///
#[derive(Error, Debug)]
pub enum GeoError {
    #[error("Invalid coordinates")]
    InvalidCoordinates,
}
