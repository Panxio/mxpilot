pub mod error;
pub mod features;
pub mod frame;
pub mod transport;
pub mod device;
pub mod hidraw;
pub mod scanner;

pub use hidraw::HidrawTransport;
pub use scanner::{scan_hidraw_devices, DetectedHidDevice};

pub use error::HidppError;
pub use features::{
    BatteryInfo, BatteryState, KeyInfo, DivertedButtonsEvent,
    CID_TOP_BUTTON, CID_BACK_BUTTON, CID_FORWARD_BUTTON,
    CID_LEFT_CLICK, CID_RIGHT_CLICK, CID_MIDDLE_CLICK,
};
pub use frame::HidppFrame;
pub use transport::{Transport, MockTransport};
pub use device::HidppDevice;

