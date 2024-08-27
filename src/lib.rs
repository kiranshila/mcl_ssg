use hidapi::{HidDevice, HidError};
use interrupts::{max_freq, max_power, min_freq, min_power};
use std::{marker::PhantomData, string::FromUtf8Error};

mod interrupts;

// Factory-values for the VID and PID for all SSG devices
const VID: u16 = 0x20CE;
const PID: u16 = 0x0012;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Could not parse string")]
    InvalidString(#[from] FromUtf8Error),
    #[error("Lower-level HID error")]
    Hid(#[from] HidError),
    #[error("Tried to construct an instance of the wrong device type")]
    WrongDevice,
    #[error("HID interrupt returned a different code than we expected")]
    BadHidRead,
    #[error("Requested value was out of range")]
    OutOfRange,
}

mod marker {
    /// Marker type for the SSG-6000 series devices
    pub struct Ssg6000;
    /// Marker type for the SSG-XG series devices
    pub struct SsgXg;
}
pub use marker::*;

pub struct MclSsg<T> {
    dev: HidDevice,
    model: PhantomData<T>,
    min_freq: u64,
    max_freq: u64,
    min_power: f32,
    max_power: f32,
}

pub type MclSsgResult<T> = Result<T, Error>;

impl MclSsg<Ssg6000> {
    /// Open an SSG device. If multiple are connected, this may be non-deterministic
    pub fn new() -> MclSsgResult<Self> {
        let api = hidapi::HidApi::new()?;
        let dev = api.open(VID, PID)?;
        let model = interrupts::model_name(&dev)?;
        let min_freq = min_freq(&dev)?;
        let max_freq = max_freq(&dev)?;
        let min_power = min_power(&dev)?;
        let max_power = max_power(&dev)?;
        if model.starts_with("SSG-6000") {
            Ok(Self {
                dev,
                model: PhantomData,
                min_freq,
                max_freq,
                min_power,
                max_power,
            })
        } else {
            Err(Error::WrongDevice)
        }
    }
}

#[derive(Debug)]
pub struct Status {
    // Is the RF output enabled
    pub enabled: bool,
    // Is the frequency locked to some reference
    pub locked: bool,
    // Frequency in Hz
    pub freq: u64,
    // Power in dBm
    pub power: f32,
}
