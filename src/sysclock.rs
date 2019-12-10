use cortex_m::peripheral::SYST;

pub struct SysClock {
    syst: SYST,
    counter: u128,
}
