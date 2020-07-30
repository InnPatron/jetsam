use super::ts::TsFlavor;

pub const TS_NUM_STRINGS: &'static [&'static str] = &[
    "ts-num",
    "TS-NUM",
];

pub const TS_FULL_STRINGS: &'static [&'static str] = &[
    "ts-full",
    "TS-FULL",
];

pub const TS_FLAVOR_STRINGS: &'static [&'static str] = &[
    "ts-num",
    "TS-NUM",

    "ts-full",
    "TS-FULL",
];

pub const DEFAULT_TS_FLAVOR: (TsFlavor, &'static str) = (TsFlavor::TsNum, "TS-NUM");
