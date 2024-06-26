//! Communications module.


/// Message priority.
///
/// These are mostly used to control message arbitration and not necessarily
/// ever read.
#[derive(Default, Copy, Clone)]
pub enum Priority {
    /// Messages that may be safety critical in delivery.
    Critical = 0,

    /// Messages that should have guarenteed delivery.
    Important = 1,

    /// Messages used to assert control.
    Control = 2,

    /// Messages used for notifications.
    Notification = 3,

    /// Messages used to report status.
    Status = 4,

    // Not in use
    // Reserved = 5,
    /// Default message priority.
    #[default]
    Default = 6,

    /// Unimportant messages.
    Background = 7,
}

/// Message format identifier
#[repr(u8)]
pub enum MessageFormat {
    // broadcast messages
    /// Startup status message
    Startup = 0xF0,
    /// Heartbeat status message
    Heartbeat = 0xF1,

    // addressable messages

    // Lighting message
    Lighting = 0x42,

    // Speed message
    Speed = 0x95, // kachow

    Battery = 0xBB,

    Temperature = 0x73,

    // Driver control type (For torque or cruise modes)
    ControlType = 0xC7,

    DriverMode = 0xD7,

    /// Generic reset command message
    Reset = 0x00,
    /// Generic enable command message
    Enable = 0x01,
    /// Generic disable command message
    Disable = 0x02,
}

// This is a hack to allow conversion to and from integers. There are a few
// crates that do this witha macro but i'm unsure if i trust them.
impl TryFrom<u8> for MessageFormat {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        const STARTUP: u8 = MessageFormat::Startup as u8;
        const HEARTBEAT: u8 = MessageFormat::Heartbeat as u8;
        const RESET: u8 = MessageFormat::Reset as u8;
        const ENABLE: u8 = MessageFormat::Enable as u8;
        const DISABLE: u8 = MessageFormat::Disable as u8;

        match value {
            STARTUP => Ok(MessageFormat::Startup),
            HEARTBEAT => Ok(MessageFormat::Heartbeat),
            RESET => Ok(MessageFormat::Reset),
            ENABLE => Ok(MessageFormat::Enable),
            DISABLE => Ok(MessageFormat::Disable),

            _ => Err("failed to convert"),
        }
    }
}

/// Group extension identifier
#[repr(u8)]
pub enum GroupExtension {
    Default = 0x00,
}
