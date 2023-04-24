use serde::{Serialize, Deserialize};
use crate::device::input::InputLayout;

/// Unique data that differentiates a certain device from any other
#[derive(Serialize, Deserialize, Clone, Debug, Hash)]
pub struct DeviceIdentifier {
    /// Driver that defined the device
    pub driver_name: String,

    /// Identifier used for the device, eg. serial number
    pub identifier: String,

    /// Short description of the device, eg. "Elgato Stream Deck Plus"
    pub description: String,
}

/// Metadata describing the device
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DeviceMetadata {
    /// Driver that found the device
    pub identifier: DeviceIdentifier,

    /// Input layout of the device
    pub layout: InputLayout,
}