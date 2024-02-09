# imxrt-flash-algorithm

This is a [CMSIS Pack compatible flash algorithm](https://open-cmsis-pack.github.io/Open-CMSIS-Pack-Spec/main/html/algorithmFunc.html) for the [NXP i.MX RT Series ARM MCUs](https://www.nxp.com/products/processors-and-microcontrollers/arm-microcontrollers/i-mx-rt-crossover-mcus:IMX-RT-SERIES?tid=vanIMXRT).

Known compatible products:

- [i.MX RT1060](https://www.nxp.com/products/processors-and-microcontrollers/arm-microcontrollers/i-mx-rt-crossover-mcus/i-mx-rt1060-crossover-mcu-with-arm-cortex-m7:i.MX-RT1060)

# Features

There are two Cargo features:

- `log` - adds support for logging over UART (useful for debugging the flash algorithm)
- `miniz` - enables support for `probe-rs`'s [`miniz` transfer encoding](https://github.com/probe-rs/probe-rs/pull/1947) (enabled by default)

The `miniz` feature is a departure from the CMSIS standard, so enabling it will result in an algorithm that *only* works with `probe-rs`. The `miniz` transfer encoding reduces programming time by around 45% on my projects.

# License

This project is licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)

- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
