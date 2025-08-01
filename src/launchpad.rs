pub(crate) struct Launchpad {
    pub name: &'static str,
    pub program: &'static str,
    pub discriminator: &'static [u8],
    pub account_idx: usize,
}

pub(crate) const RAYDIUM: Launchpad = Launchpad {
    name: "Raydium",
    program: "LanMV9sAd7wArD4vJFi2qDdfnVhFxYSUg6eADduJ3uj",
    discriminator: &[175, 175, 109, 31, 13, 152, 155, 237],
    account_idx: 6,
};
pub(crate) const PUMP_FUN: Launchpad = Launchpad {
    name: "Pump.fun",
    program: "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P",
    discriminator: &[24, 30, 200, 40, 5, 28, 7, 119],
    account_idx: 0,
};