///
/// Charging connector types.
///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectorType {
    Ccs1,
    Ccs2,
    Chademo,
    Gbt,
    Nacs,
}
