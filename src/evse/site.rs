use crate::containers::ChargeProfileList;
use crate::errors::SimulationError;
use crate::evse::{Charger, ChargerState};
use rustc_hash::FxHashMap;
use uuid::Uuid;

///
/// Object that represents an EV charging site.
///
#[derive(Debug)]
pub struct Site {
    pub id: Uuid,
    pub name: String,
    pub chargers: Vec<Charger>,
    pub site_max_power_kw: Option<f64>,
    charger_map: FxHashMap<Uuid, usize>,
}

impl Site {
    ///
    /// Create a new EV charging site from a list of chargers.
    ///
    pub fn new(name: &str, chargers: Vec<Charger>) -> Self {
        let charger_map = chargers
            .iter()
            .enumerate()
            .map(|(i, c)| (c.id, i))
            .collect();

        Self {
            id: Uuid::new_v4(),
            name: name.to_owned(),
            chargers,
            site_max_power_kw: None,
            charger_map,
        }
    }

    ///
    /// Set the total maximum power (kW) available at the site.
    ///
    pub fn site_max_power_kw(&mut self, val: f64) -> &mut Self {
        self.site_max_power_kw = Some(val);
        self
    }

    ///
    /// Get the number of chargers on site.
    ///
    pub fn num_chargers(&self) -> usize {
        self.chargers.len()
    }

    ///
    /// Get the total site power.
    /// Note that this might be larger than the site maximum power.
    ///
    pub fn site_power_kw(&self) -> f64 {
        self.chargers.iter().map(|c| c.max_power_kw).sum()
    }

    ///
    /// Get the number of occupied chargers on site.
    ///
    pub fn num_occupied(&self) -> usize {
        self.chargers.iter().filter(|c| c.is_busy).count()
    }

    ///
    /// Get the first unoccupied charger.
    /// Returns None if all chargers are occupied.
    ///
    pub fn get_unoccupied_charger(&self) -> Option<&Charger> {
        self.chargers.iter().find(|c| !c.is_busy)
    }

    ///
    /// Get the first unoccupied charger as mutable.
    /// Returns None if all chargers are occupied.
    ///
    pub fn get_unoccupied_charger_mut(&mut self) -> Option<&mut Charger> {
        self.chargers.iter_mut().find(|c| !c.is_busy)
    }

    ///
    /// Get a charger from the site using its ID.
    ///
    pub fn get_charger(&self, id: &Uuid) -> Result<&Charger, SimulationError> {
        self.charger_map
            .get(id)
            .map(|&idx| &self.chargers[idx])
            .ok_or_else(|| SimulationError::InvalidChargerId(id.to_string()))
    }

    ///
    /// Get a mutable charger from the site using its ID.
    ///
    pub fn get_charger_mut(&mut self, id: &Uuid) -> Result<&mut Charger, SimulationError> {
        self.charger_map
            .get(id)
            .map(|&idx| &mut self.chargers[idx])
            .ok_or_else(|| SimulationError::InvalidChargerId(id.to_string()))
    }

    ///
    /// Allocate the requested power and maximum power each currently
    /// active charger can deliver in the current timestep.
    ///
    pub fn allocate_power(
        &self,
        active_charger_states: &mut FxHashMap<Uuid, ChargerState>,
        charge_profile_list: &ChargeProfileList,
    ) -> Result<(), SimulationError> {
        // Get requested power for each active charger and sum total requested power at the site
        let mut total_requested_power_kw = 0.0;
        for (charger_id, charger_state) in active_charger_states.iter_mut() {
            let charger = self.get_charger(charger_id)?;
            let charger_max_power_kw = charger.actual_max_power_kw();

            let charge_profile =
                charge_profile_list.get_charge_profile(&charger_state.charge_profile_id)?;

            let requested_power_kw = charge_profile.power_at(charger_state.current_soc)?;

            total_requested_power_kw += requested_power_kw.min(charger_max_power_kw);
            charger_state.requested_power_kw = requested_power_kw;
            charger_state.charger_max_power_kw = charger_max_power_kw;
        }

        // If the total requested power is more than the site limit then scale all
        // active charger powers by the ratio of the site limit to the requested power
        let scaling_factor = match self.site_max_power_kw {
            Some(site_max_power_kw) if total_requested_power_kw > site_max_power_kw => {
                site_max_power_kw / total_requested_power_kw
            }
            _ => 1.0,
        };
        for charger_state in active_charger_states.values_mut() {
            charger_state.max_power_kw = if scaling_factor < 1.0 {
                charger_state
                    .requested_power_kw
                    .min(charger_state.charger_max_power_kw)
                    * scaling_factor
            } else {
                charger_state.charger_max_power_kw
            };
        }
        Ok(())
    }
}
