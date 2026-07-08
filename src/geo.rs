//!
//! Geolocation functions.
//!
use crate::errors::GeoError;
use uuid::Uuid;

// Mean earth radius in miles
const EARTH_RADIUS: f64 = 3959.0;

///
/// Anything with a fixed geolocation
/// (such as an EV site or an origin or destination location).
///
pub trait Located {
    fn coordinates(&self) -> &Coords;

    ///
    /// Calculate the Haversine distance to another location.
    ///
    fn distance_to(&self, to: impl Located) -> f64 {
        self.coordinates().distance_to(to.coordinates())
    }

    ///
    /// Calculate the time to another location based on average driving speed.
    ///
    fn time_to(&self, to: impl Located, avg_speed: f64) -> f64 {
        self.coordinates().time_to(to.coordinates(), avg_speed)
    }

    ///
    /// Calculate the energy consumed driving to another location from
    /// EV consumption rate (EVC).
    ///
    fn energy_to(&self, to: impl Located, evc: f64) -> f64 {
        self.coordinates().energy_to(to.coordinates(), evc)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Coords {
    pub latitude: f64,
    pub longitude: f64,
}

impl Coords {
    ///
    /// Create a new Coords object.
    ///
    pub fn try_new(latitude: f64, longitude: f64) -> Result<Self, GeoError> {
        if (-90.0..=90.0).contains(&latitude) && (-180.0..=180.0).contains(&longitude) {
            Ok(Self {
                latitude,
                longitude,
            })
        } else {
            Err(GeoError::InvalidCoordinates)
        }
    }

    ///
    /// Calculate the Haversine distance to another Coord.
    ///
    pub fn distance_to(&self, to: &Coords) -> f64 {
        haversine(self, to)
    }

    ///
    /// Calculate the time to another Coord based on average driving speed.
    ///
    pub fn time_to(&self, to: &Coords, avg_speed: f64) -> f64 {
        haversine(self, to) / avg_speed
    }

    ///
    /// Calculate the energy consumed driving to another Coord from
    /// EV consumption rate (EVC).
    ///
    pub fn energy_to(&self, to: &Coords, evc: f64) -> f64 {
        haversine(self, to) * evc
    }
}

///
/// The Haversine function.
///
fn haversine_fn(theta: f64) -> f64 {
    (1.0 - theta.cos()) / 2.0
}

///
/// Haversine distance between two points in miles.
///
pub fn haversine(start: &Coords, end: &Coords) -> f64 {
    let phi1 = start.latitude.to_radians();
    let phi2 = end.latitude.to_radians();
    let lambda1 = start.longitude.to_radians();
    let lambda2 = end.longitude.to_radians();
    let hav = haversine_fn(phi2 - phi1) + phi1.cos() * phi2.cos() * haversine_fn(lambda2 - lambda1);

    2.0 * EARTH_RADIUS * hav.sqrt().asin()
}

///
/// A single origin or destination location.
///
#[derive(Debug)]
pub struct Location {
    pub id: Uuid,
    pub name: String,
    coordinates: Coords,
}

impl Location {
    ///
    /// Create a new Location from its coordinates.
    ///
    pub fn new(name: &str, coordinates: Coords) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_owned(),
            coordinates,
        }
    }
}

impl Located for Location {
    fn coordinates(&self) -> &Coords {
        &self.coordinates
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn test_distance_to() {
        let loc1 = Location::new("loc1", Coords::try_new(37.78055877, -122.47253935).unwrap());
        let loc2 = Location::new("loc2", Coords::try_new(37.73773718, -122.40792795).unwrap());
        let d = loc1.distance_to(loc2);

        assert_approx_eq!(d, 4.6057475);
    }

    #[test]
    fn test_time_to() {
        let loc1 = Location::new("loc1", Coords::try_new(37.78055877, -122.47253935).unwrap());
        let loc2 = Location::new("loc2", Coords::try_new(37.73773718, -122.40792795).unwrap());
        let t = loc1.time_to(loc2, 20.0);

        assert_approx_eq!(t, 0.230287375);
    }

    #[test]
    fn test_energy_to() {
        let loc1 = Location::new("loc1", Coords::try_new(37.78055877, -122.47253935).unwrap());
        let loc2 = Location::new("loc2", Coords::try_new(37.73773718, -122.40792795).unwrap());
        let e = loc1.energy_to(loc2, 0.25);

        assert_approx_eq!(e, 1.151436875);
    }
}
