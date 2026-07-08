use crate::errors::SimulationError;
use crate::geo::Location;
use rustc_hash::FxHashMap;
use uuid::Uuid;

///
/// A list of origin or destination locations in a region.
///
#[derive(Debug, Default)]
pub struct LocationList {
    locations: Vec<Location>,
    location_map: FxHashMap<Uuid, usize>,
}

impl LocationList {
    ///
    /// Create a new list of locations.
    ///
    pub fn new(locations: Vec<Location>) -> Self {
        let location_map = locations
            .iter()
            .enumerate()
            .map(|(i, loc)| (loc.id, i))
            .collect();

        Self {
            locations,
            location_map,
        }
    }

    ///
    /// Get the number of locations.
    ///
    pub fn num_locations(&self) -> usize {
        self.locations.len()
    }

    ///
    /// Fetch a location from the list using its ID.
    ///
    pub fn get_location(&self, id: &Uuid) -> Result<&Location, SimulationError> {
        self.location_map
            .get(id)
            .map(|&idx| &self.locations[idx])
            .ok_or_else(|| SimulationError::InvalidLocationId(id.to_string()))
    }
}

impl FromIterator<Location> for LocationList {
    fn from_iter<I: IntoIterator<Item = Location>>(iter: I) -> Self {
        Self::new(iter.into_iter().collect())
    }
}

impl Extend<Location> for LocationList {
    fn extend<I: IntoIterator<Item = Location>>(&mut self, iter: I) {
        for location in iter {
            let idx = self.locations.len();
            self.location_map.insert(location.id, idx);
            self.locations.push(location);
        }
    }
}
