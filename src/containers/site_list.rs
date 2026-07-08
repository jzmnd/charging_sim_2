use crate::errors::SimulationError;
use crate::evse::Site;
use rustc_hash::FxHashMap;
use uuid::Uuid;

///
/// A list of EV charging sites in a region.
///
#[derive(Debug, Default)]
pub struct SiteList {
    sites: Vec<Site>,
    site_map: FxHashMap<Uuid, usize>,
}

impl SiteList {
    ///
    /// Create a new list of sites.
    ///
    pub fn new(sites: Vec<Site>) -> Self {
        let site_map = sites.iter().enumerate().map(|(i, s)| (s.id, i)).collect();

        Self { sites, site_map }
    }

    ///
    /// Get the number of sites.
    ///
    pub fn num_sites(&self) -> usize {
        self.sites.len()
    }

    ///
    /// Fetch a site from the list using its ID.
    ///
    pub fn get_site(&self, id: &Uuid) -> Result<&Site, SimulationError> {
        self.site_map
            .get(id)
            .map(|&idx| &self.sites[idx])
            .ok_or_else(|| SimulationError::InvalidSiteId(id.to_string()))
    }
}

impl FromIterator<Site> for SiteList {
    fn from_iter<I: IntoIterator<Item = Site>>(iter: I) -> Self {
        Self::new(iter.into_iter().collect())
    }
}

impl Extend<Site> for SiteList {
    fn extend<I: IntoIterator<Item = Site>>(&mut self, iter: I) {
        for site in iter {
            let idx = self.sites.len();
            self.site_map.insert(site.id, idx);
            self.sites.push(site);
        }
    }
}
