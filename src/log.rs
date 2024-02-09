use core::convert::Infallible;

// for serial Write trait
use cortex_m::prelude::*;

use imxrt_hal as hal;
use imxrt_ral as ral;

use ufmt_write::uWrite;

pub const XTAL_OSCILLATOR_FREQUENCY: u32 = 24_000_000;
const UART_DIVIDER: u32 = 3;
pub const UART_FREQUENCY: u32 = XTAL_OSCILLATOR_FREQUENCY / UART_DIVIDER;

// The lpuart number and TX/RX pads.
// These particular pads correspond to the pins labeled TX and RX on the MicroMod ATP board
const LPUART_INSTANCE: u8 = 6;
type TxPad = hal::iomuxc::pads::gpio_ad_b0::GPIO_AD_B0_02;
type RxPad = hal::iomuxc::pads::gpio_ad_b0::GPIO_AD_B0_03;

const BAUD: u32 = 115200;

static mut CONSOLE: Option<hal::lpuart::Lpuart<hal::lpuart::Pins<TxPad, RxPad>, LPUART_INSTANCE>> =
    None;

pub struct UARTWriter;

impl uWrite for UARTWriter {
    type Error = Infallible;

    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
        for c in s.bytes() {
            unsafe {
                let _ = nb::block!(CONSOLE.as_mut().unwrap_unchecked().write(c));
            }
        }

        Ok(())
    }
}

#[allow(dead_code)]
pub const LOG: UARTWriter = UARTWriter;

#[allow(dead_code)]
pub unsafe fn init() {
    if CONSOLE.is_none() {
        let mut instances = ral::Instances::instances();

        // configure UART clock
        hal::ccm::clock_gate::lpuart::<LPUART_INSTANCE>()
            .set(&mut instances.CCM, hal::ccm::clock_gate::OFF);
        hal::ccm::uart_clk::set_selection(
            &mut instances.CCM,
            hal::ccm::uart_clk::Selection::Oscillator,
        );
        hal::ccm::uart_clk::set_divider(&mut instances.CCM, UART_DIVIDER);
        hal::ccm::clock_gate::lpuart::<LPUART_INSTANCE>()
            .set(&mut instances.CCM, hal::ccm::clock_gate::ON);

        let pads = hal::iomuxc::pads::Pads::new();
        let tx: TxPad = pads.gpio_ad_b0.p02;
        let rx: RxPad = pads.gpio_ad_b0.p03;
        let console = lpuart::<_, _, LPUART_INSTANCE>(instances.LPUART6, tx, rx, BAUD);

        CONSOLE = Some(console);
    }
}

pub fn lpuart<Tx, Rx, const N: u8>(
    instance: ral::lpuart::Instance<N>,
    tx: Tx,
    rx: Rx,
    baud: u32,
) -> hal::lpuart::Lpuart<hal::lpuart::Pins<Tx, Rx>, N>
where
    Tx: hal::iomuxc::lpuart::Pin<
        Direction = hal::iomuxc::lpuart::Tx,
        Module = hal::iomuxc::consts::Const<N>,
    >,
    Rx: hal::iomuxc::lpuart::Pin<
        Direction = hal::iomuxc::lpuart::Rx,
        Module = hal::iomuxc::consts::Const<N>,
    >,
{
    let baud = hal::lpuart::Baud::compute(UART_FREQUENCY, baud);
    let mut uart = hal::lpuart::Lpuart::new(instance, hal::lpuart::Pins { tx, rx });
    uart.disable(|uart| uart.set_baud(&baud));
    uart
}
