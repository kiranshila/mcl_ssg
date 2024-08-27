use hidapi::HidDevice;

use crate::{MclSsg, MclSsgResult};
const SEND_PACKET_LEN: usize = 64;

#[repr(u8)]
enum InterruptCode {
    DeviceModelName = 40,
    DeviceSerialNumber = 41,
    GeneratorMinimumFrequency = 42,
    GeneratorMaximumFrequency = 43,
    GeneratorMinimumPower = 45,
    GeneratorMaximumPower = 46,
    SetFreqAndPower = 103,
    SetRfPowerOnOff = 104,
    GetGeneratorOutputStatus = 105,
}

macro_rules! pack_with_interrupt {
    ($code:ident) => {{
        let mut bytes = [0u8; SEND_PACKET_LEN];
        bytes[0] = InterruptCode::$code as u8;
        bytes
    }};
}

/// Write then read, validating the return interrupt code
fn write_read(dev: &HidDevice, bytes: &mut [u8]) -> MclSsgResult<()> {
    let code = bytes[0];
    dev.write(bytes)?;
    dev.read(bytes)?;
    if bytes[0] != code {
        Err(super::Error::BadHidRead)
    } else {
        Ok(())
    }
}

/// Internal method to validate device type
pub(crate) fn model_name(dev: &HidDevice) -> MclSsgResult<String> {
    let mut bytes = pack_with_interrupt!(DeviceModelName);
    write_read(dev, &mut bytes)?;
    let null_idx = bytes
        .iter()
        .position(|x| *x == 0)
        .expect("cstr must contain null");
    Ok(String::from_utf8(bytes[1..null_idx].to_vec())?)
}

pub(crate) fn min_freq(dev: &HidDevice) -> MclSsgResult<u64> {
    let mut bytes = pack_with_interrupt!(GeneratorMinimumFrequency);
    write_read(dev, &mut bytes)?;
    let mut freq_bytes = [0u8; 8];
    freq_bytes[4..].clone_from_slice(&bytes[1..5]);
    let freq = u64::from_be_bytes(freq_bytes);
    Ok(freq)
}

pub(crate) fn max_freq(dev: &HidDevice) -> MclSsgResult<u64> {
    let mut bytes = pack_with_interrupt!(GeneratorMaximumFrequency);
    write_read(dev, &mut bytes)?;
    let mut freq_bytes = [0u8; 8];
    freq_bytes[3..].clone_from_slice(&bytes[1..6]);
    let freq = u64::from_be_bytes(freq_bytes);
    Ok(freq)
}

pub(crate) fn min_power(dev: &HidDevice) -> MclSsgResult<f32> {
    let mut bytes = pack_with_interrupt!(GeneratorMinimumPower);
    write_read(dev, &mut bytes)?;
    let power = bytes_to_power(&bytes[1..4]);
    Ok(power)
}

pub(crate) fn max_power(dev: &HidDevice) -> MclSsgResult<f32> {
    let mut bytes = pack_with_interrupt!(GeneratorMaximumPower);
    write_read(dev, &mut bytes)?;
    let power = bytes_to_power(&bytes[1..4]);
    Ok(power)
}

fn bytes_to_power(bytes: &[u8]) -> f32 {
    (-1_f32).powf(bytes[0] as f32) * (256f32 * bytes[1] as f32 + bytes[2] as f32) / 100f32
}

fn power_to_bytes(power: f32) -> Vec<u8> {
    let mut bytes = vec![0u8; 3];
    bytes[0] = (power.signum() == -1.0) as u8;
    let power1 = (power.abs() * 100.0 / 256.0).trunc();
    let power2 = power.abs() * 100.0 - power1 * 256.0;
    bytes[1] = power1 as u8;
    bytes[2] = power2 as u8;
    bytes
}

/// Implementations for the generic SSG
impl<T> MclSsg<T> {
    /// Get the connected generator's model name
    pub fn get_model_name(&self) -> MclSsgResult<String> {
        model_name(&self.dev)
    }

    /// Get the connected generator's serial number
    pub fn get_serial_number(&self) -> MclSsgResult<String> {
        let mut bytes = pack_with_interrupt!(DeviceSerialNumber);
        write_read(&self.dev, &mut bytes)?;
        let null_idx = bytes
            .iter()
            .position(|x| *x == 0)
            .expect("cstr must contain null");
        Ok(String::from_utf8(bytes[1..null_idx].to_vec())?)
    }

    /// Get the output status of the signal generator
    pub fn get_status(&self) -> MclSsgResult<super::Status> {
        let mut bytes = pack_with_interrupt!(GetGeneratorOutputStatus);
        write_read(&self.dev, &mut bytes)?;
        let enabled = bytes[1] != 0;
        let locked = bytes[2] != 0;
        let mut freq_bytes = [0u8; 8];
        freq_bytes[3..].clone_from_slice(&bytes[3..8]);
        let freq = u64::from_be_bytes(freq_bytes);
        let power = bytes_to_power(&bytes[8..11]);
        Ok(super::Status {
            enabled,
            locked,
            freq,
            power,
        })
    }

    /// Enables or disables the output RF power
    pub fn set_rf_power_on(&self, enabled: bool) -> MclSsgResult<()> {
        let mut bytes = pack_with_interrupt!(SetRfPowerOnOff);
        bytes[1] = enabled as u8;
        write_read(&self.dev, &mut bytes)?;
        Ok(())
    }

    /// Get the minimum supported frequency in Hz
    pub fn get_min_freq(&self) -> u64 {
        self.min_freq
    }

    /// Get the maximum supported frequency in Hz
    pub fn get_max_freq(&self) -> u64 {
        self.max_freq
    }

    /// Get the minimum supported power in dBm
    pub fn get_min_power(&self) -> f32 {
        self.min_power
    }

    /// Get the maximum supported power in dBm
    pub fn get_max_power(&self) -> f32 {
        self.max_power
    }
}

/// Implementations for the SSG-6000 series
impl MclSsg<super::Ssg6000> {
    /// Set the RF output frequency in Hz, power in dBm, and the trigger out function
    pub fn set_frequency_power_trigger(
        &self,
        freq: u64,
        power: f32,
        trigger: bool,
    ) -> MclSsgResult<()> {
        if freq < self.min_freq
            || freq > self.max_freq
            || power < self.min_power
            || power > self.max_power
        {
            return Err(super::Error::OutOfRange);
        }
        let mut bytes = pack_with_interrupt!(SetFreqAndPower);
        let freq_bytes = freq.to_be_bytes();
        bytes[1..6].clone_from_slice(&freq_bytes[3..]);
        let power_bytes = power_to_bytes(power);
        bytes[6..9].clone_from_slice(&power_bytes);
        bytes[9] = trigger as u8;
        write_read(&self.dev, &mut bytes)?;
        Ok(())
    }
}
