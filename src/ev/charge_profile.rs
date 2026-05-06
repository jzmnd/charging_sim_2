use crate::errors::SimulationError;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize, PartialEq, PartialOrd)]
pub struct ChargeProfileRecord {
    pub soc: f64,
    pub power: f64,
}

#[derive(Debug, Default)]
pub struct ChargingOutput {
    pub duration_s: f64,
    pub energy_kwh: f64,
    pub peak_power_kw: f64,
}

#[derive(Debug)]
pub struct ChargeProfile {
    pub id: Uuid,
    pub name: String,
    data: Vec<ChargeProfileRecord>,
    battery_capacity_kwh: f64,
}

impl ChargeProfile {
    ///
    /// Create a charge profile from SOC vs power data.
    ///
    pub fn new(name: &str, mut data: Vec<ChargeProfileRecord>, battery_capacity_kwh: f64) -> Self {
        data.sort_by(|a, b| {
            a.soc
                .partial_cmp(&b.soc)
                .expect("SOC values should never contain NaN")
        });
        // extrapolate to SOC = 0 if not present in the data
        if let Some(first) = data.first()
            && first.soc > 0.0
        {
            data.insert(
                0,
                ChargeProfileRecord {
                    soc: 0.0,
                    power: first.power,
                },
            );
        }
        // extrapolate to SOC = 1 if not present in the data
        if let Some(last) = data.last()
            && last.soc < 1.0
        {
            data.push(ChargeProfileRecord {
                soc: 1.0,
                power: last.power,
            });
        }
        Self {
            id: Uuid::new_v4(),
            name: name.to_owned(),
            data,
            battery_capacity_kwh,
        }
    }

    ///
    /// Create a charge profile from a file of SOC vs power data.
    ///
    pub fn from_file(
        name: &str,
        filename: &str,
        battery_capacity_kwh: f64,
    ) -> Result<Self, csv::Error> {
        let mut data: Vec<ChargeProfileRecord> = Vec::new();
        let mut reader = csv::Reader::from_path(filename)?;
        for record in reader.deserialize() {
            data.push(record?);
        }
        Ok(Self::new(name, data, battery_capacity_kwh))
    }

    fn binary_search_soc(&self, soc: f64) -> Option<usize> {
        let idx = self.data.binary_search_by(|r| {
            r.soc
                .partial_cmp(&soc)
                .expect("SOC values should never contain NaN")
        });

        match idx {
            Ok(i) => Some(i),
            Err(i) => i.checked_sub(1),
        }
    }

    ///
    /// Return the closest power for a given SOC.
    ///
    pub fn power_at(&self, soc: f64) -> Result<f64, SimulationError> {
        let idx_max = self.data.len().saturating_sub(1);

        match self.binary_search_soc(soc) {
            None => Ok(self
                .data
                .first()
                .ok_or(SimulationError::EmptyChargeProfile)?
                .power),
            Some(i) if i >= idx_max => Ok(self
                .data
                .last()
                .ok_or(SimulationError::EmptyChargeProfile)?
                .power),
            Some(i) => {
                let r0 = &self.data[i];
                let r1 = &self.data[i + 1];
                Ok(r0.power + (soc - r0.soc) * (r1.power - r0.power) / (r1.soc - r0.soc))
            }
        }
    }

    ///
    /// Calculate the properties of a charging event by integrating over
    /// the charge profile from a starting to target SOC.
    ///
    pub fn calculate(
        &self,
        soc_start: f64,
        soc_target: f64,
        max_power_kw: f64,
    ) -> Result<ChargingOutput, SimulationError> {
        let mut output = ChargingOutput::default();
        let mut soc = soc_start;

        for segment in self.data.windows(2) {
            let seg_start = &segment[0];
            let seg_end = &segment[1];
            if soc >= soc_target {
                // SOC has reached the target so stop integration.
                break;
            }
            if soc >= seg_end.soc {
                // SOC is larger than the current segment, therefore continue to next segment.
                continue;
            }
            let soc_seg_start = soc.max(seg_start.soc);
            let soc_seg_end = soc_target.min(seg_end.soc);

            let delta_soc = soc_seg_end - soc_seg_start;
            let seg_energy_kwh = delta_soc * self.battery_capacity_kwh;

            let power_start = self.power_at(soc_seg_start)?.min(max_power_kw);
            let power_end = self.power_at(soc_seg_end)?.min(max_power_kw);
            let avg_power = 0.5 * (power_start + power_end);
            if avg_power > output.peak_power_kw {
                output.peak_power_kw = avg_power;
            }

            let seg_duration_s = 60.0 * 60.0 * seg_energy_kwh / avg_power;

            output.energy_kwh += seg_energy_kwh;
            output.duration_s += seg_duration_s;

            // Update the new SOC to be the end of the segment.
            soc = soc_seg_end;
        }
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn test_power_at() {
        let charge_profile =
            ChargeProfile::from_file("cp1", "data/example_charge_profile_1.csv", 60.0).unwrap();
        assert_approx_eq!(charge_profile.power_at(0.01).unwrap(), 210.3);
        assert_approx_eq!(charge_profile.power_at(0.50).unwrap(), 96.19310344827586);
        assert_approx_eq!(charge_profile.power_at(0.99).unwrap(), 32.6);
    }

    #[test]
    fn test_calculate() {
        let charge_profile =
            ChargeProfile::from_file("cp1", "data/example_charge_profile_1.csv", 60.0).unwrap();
        let output = charge_profile.calculate(0.2, 0.6, 150.0).unwrap();
        assert_approx_eq!(output.duration_s, 732.4643232975575);
        assert_approx_eq!(output.energy_kwh, 24.0);
        assert_approx_eq!(output.peak_power_kw, 150.0);
    }

    #[test]
    fn test_calculate_full_charge() {
        let charge_profile =
            ChargeProfile::from_file("cp1", "data/example_charge_profile_1.csv", 60.0).unwrap();
        let output = charge_profile.calculate(0.0, 1.0, 150.0).unwrap();
        assert_approx_eq!(output.energy_kwh, 60.0);
        assert_approx_eq!(output.peak_power_kw, 150.0);
    }
}
