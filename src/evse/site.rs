use crate::evse::Charger;
use std::collections::HashMap;
use uuid::Uuid;

///
/// Object that represents an EV charging site.
///
#[derive(Debug)]
pub struct Site {
    pub id: Uuid,
    pub name: String,
    pub chargers: Vec<Charger>,
    charger_map: HashMap<Uuid, usize>,
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
            charger_map,
        }
    }

    ///
    /// Get the number of chargers on site.
    ///
    pub fn num_chargers(&self) -> usize {
        self.chargers.len()
    }

    ///
    /// Get the total site power.
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
    ///
    pub fn get_unoccupied_charger(&self) -> Option<&Charger> {
        self.chargers.iter().find(|c| !c.is_busy)
    }

    ///
    /// Get the first unoccupied charger as mutable.
    ///
    pub fn get_unoccupied_charger_mut(&mut self) -> Option<&mut Charger> {
        self.chargers.iter_mut().find(|c| !c.is_busy)
    }

    ///
    /// Get a charger from the site using its ID.
    ///
    pub fn get_charger(&self, id: &Uuid) -> Option<&Charger> {
        self.charger_map.get(id).map(|&idx| &self.chargers[idx])
    }

    ///
    /// Get a mutable charger from the site using its ID.
    ///
    pub fn get_charger_mut(&mut self, id: &Uuid) -> Option<&mut Charger> {
        self.charger_map.get(id).map(|&idx| &mut self.chargers[idx])
    }
}
