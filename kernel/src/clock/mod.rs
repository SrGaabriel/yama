use core::arch::asm;

pub struct SystemClock {]

}

pub fn read_tsc() -> u64 {
    let low: u32;
    let high: u32;

    unsafe {
        asm!(
            "rdtsc",
            out("eax") low,
            out("edx") high
        )
    }

    ((high as u64) << 32) | (low as u64)
}