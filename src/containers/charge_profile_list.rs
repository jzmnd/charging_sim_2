use crate::ev::ChargeProfile;
use std::collections::HashMap;
use uuid::Uuid;

///
/// A list of charge profiles including a mapping of charge profile ID to charge profile objects.
///
#[derive(Debug)]
pub struct ChargeProfileList {
    pub id: Uuid,
    pub name: String,
    charge_profiles: Vec<ChargeProfile>,
    charge_profile_map: HashMap<Uuid, usize>,
}

impl ChargeProfileList {
    ///
    /// Create a new list of charge profiles.
    ///
    pub fn new(name: &str, charge_profiles: Vec<ChargeProfile>) -> Self {
        let charge_profile_map = charge_profiles
            .iter()
            .enumerate()
            .map(|(i, c)| (c.id, i))
            .collect();

        Self {
            id: Uuid::new_v4(),
            name: name.to_owned(),
            charge_profiles,
            charge_profile_map,
        }
    }

    ///
    /// Fetch a charge profile from the list using its ID.
    ///
    pub fn get_charge_profile(&self, id: &Uuid) -> Option<&ChargeProfile> {
        self.charge_profile_map
            .get(id)
            .map(|&idx| &self.charge_profiles[idx])
    }

    ///
    /// Fetch a charge profile from the list using its name.
    ///
    pub fn get_id_by_name(&self, name: &str) -> Option<Uuid> {
        self.charge_profiles
            .iter()
            .find(|c| c.name == name)
            .map(|c| c.id)
    }

    ///
    /// Fetch all charge profile IDs.
    ///
    pub fn all_ids(&self) -> Vec<Uuid> {
        self.charge_profile_map.keys().cloned().collect()
    }
}
