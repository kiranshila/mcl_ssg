# mcl_ssg

A pure-Rust implementation of the Mini-Circuits Synthesized Signal Generator (SSG) USB interface.

```rust
use mcl_ssg::MclSsg;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ssg = MclSsg::new()?;
    println!(
        "PN: {}\nSN: {}\nMin Freq: {} Hz\nMax Freq: {} Hz\nMin Power {} dBm\nMax Power {} dBm",
        ssg.get_model_name()?,
        ssg.get_serial_number()?,
        ssg.get_min_freq(),
        ssg.get_max_freq(),
        ssg.get_min_power(),
        ssg.get_max_power()
    );
    ssg.set_frequency_power_trigger(6_000_000_000, -60.0, true)?;
    ssg.set_rf_power_on(false)?;
    let s = ssg.get_status()?;
    dbg!(s);
    Ok(())
}
```
