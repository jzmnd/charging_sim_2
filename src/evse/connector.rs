use serde::Serialize;

///
/// Charging connector types.
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ConnectorType {
    #[serde(rename = "CCS1")]
    Ccs1,
    #[serde(rename = "CCS2")]
    Ccs2,
    #[serde(rename = "CHAdeMO")]
    Chademo,
    #[serde(rename = "GB-T")]
    Gbt,
    #[serde(rename = "NACS")]
    Nacs,
}
