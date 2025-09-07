use uuid::Uuid;

#[derive(Debug)]
pub struct Charger {
    pub id: Uuid,
    pub name: String,
    pub max_power_kw: f64,
    pub max_current_a: f64,
    pub voltage: f64,
    pub occupied_until: u64,
    pub is_busy: bool,
}

impl Charger {
    ///
    /// Create a new EV charger builder.
    ///
    pub fn builder() -> ChargerBuilder {
        ChargerBuilder::default()
    }
}

const DEFAULT_MAX_POWER_KW: f64 = 500.0;
const DEFAULT_MAX_CURRENT_A: f64 = 100.0;
const DEFAULT_VOLTAGE: f64 = 400.0;

#[derive(Debug, Default)]
pub struct ChargerBuilder {
    max_power_kw: Option<f64>,
    max_current_a: Option<f64>,
    voltage: Option<f64>,
}

impl ChargerBuilder {
    pub fn max_power_kw(mut self, val: f64) -> Self {
        self.max_power_kw = Some(val);
        self
    }

    pub fn max_current_a(mut self, val: f64) -> Self {
        self.max_current_a = Some(val);
        self
    }

    pub fn voltage(mut self, val: f64) -> Self {
        self.voltage = Some(val);
        self
    }

    ///
    /// Build a named charger.
    ///
    pub fn build(&self, name: &str) -> Charger {
        Charger {
            id: Uuid::new_v4(),
            name: name.to_owned(),
            max_power_kw: self.max_power_kw.unwrap_or(DEFAULT_MAX_POWER_KW),
            max_current_a: self.max_current_a.unwrap_or(DEFAULT_MAX_CURRENT_A),
            voltage: self.voltage.unwrap_or(DEFAULT_VOLTAGE),
            occupied_until: 0,
            is_busy: false,
        }
    }
}
