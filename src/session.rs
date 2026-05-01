use serde::Serialize;
use uuid::Uuid;

///
/// Charging session data.
/// Reneged sessions (vehicle left the waiting queue before being charged) and
/// balked sessions (vehicle never joined the queue because it was too long)
/// have the charger and charging data fields as `None`.
///
#[derive(Debug, Serialize)]
pub struct Session {
    pub vehicle: String,
    pub vehicle_id: Uuid,
    pub charge_profile: String,
    pub charge_profile_id: Uuid,
    pub charger: Option<String>,
    pub charger_id: Option<Uuid>,
    pub arrival_time: u64,
    pub plugin_time: Option<u64>,
    pub unplug_time: Option<u64>,
    pub wait_duration_s: u64,
    pub reneged: bool,
    pub balked: bool,
    pub charge_duration_s: Option<f64>,
    pub idle_duration_s: Option<f64>,
    pub peak_power_kw: Option<f64>,
    pub energy_kwh: Option<f64>,
    pub start_soc: f64,
    pub end_soc: Option<f64>,
}
