/// Vehicle devices
#[derive(Debug, Clone, Copy)]
pub enum Device {
    VehicleController,
    SteeringWheel,
    ArrayIsolationController,
    WaveSculptor,
    MpptA,
    MpptB,
}

/// Get J1939 source address
pub fn source_address(device: Device) -> Option<u8> {
    match device {
        Device::VehicleController => Some(0x10),
        Device::SteeringWheel => Some(0x20),
        Device::ArrayIsolationController => Some(0x30),
        _ => None,
    }
}

/// Get base address for device (for legacy devices)
pub fn base_address(device: Device) -> Option<u16> {
    match device {
        Device::WaveSculptor => Some(0x400),
        Device::MpptA => Some(0x600),
        Device::MpptB => Some(0x610),
        _ => None,
    }
}
