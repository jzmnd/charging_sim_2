use serde::Serialize;
use uuid::Uuid;

///
/// Charging session data.
///
#[derive(Debug, Serialize)]
pub struct Session {
    pub vehicle: String,
    pub vehicle_id: Uuid,
    pub charge_profile: String,
    pub charge_profile_id: Uuid,
    pub charger: String,
    pub charger_id: Uuid,
    pub arrival_time: u64,
    pub plugin_time: u64,
    pub unplug_time: u64,
    pub wait_duration_s: u64,
    pub charge_duration_s: f64,
    pub idle_duration_s: f64,
    pub max_power_kw: f64,
    pub energy_kwh: f64,
    pub start_soc: f64,
    pub end_soc: f64,
}

///
/// Save a list of sessions to .csv
///
pub fn save_to_csv(sessions: &[Session], filename: &str) -> Result<(), csv::Error> {
    let mut wtr = csv::Writer::from_path(filename)?;
    for session in sessions {
        wtr.serialize(session)?;
    }
    wtr.flush()?;
    Ok(())
}
