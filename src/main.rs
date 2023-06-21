#![no_std]
#![no_main]

use panic_halt as _;
use embedded_alloc::Heap;

use rp2040_hal as hal;
use hal::pac;

use rand_core::RngCore;

use chip8_core::Chip8;

#[global_allocator]
static HEAP: Heap = Heap::empty();

#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_GENERIC_03H;

const XTAL_FREQ_HZ: u32 = 12_000_000u32;

#[hal::entry]
fn main() -> ! {
    // initialize the heap
    {
        use core::mem::MaybeUninit;
        const HEAP_SIZE: usize = 1024;
        static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_SIZE) }
    }
    // initialize the peripherals 
    let mut pac = pac::Peripherals::take().unwrap();

    // initialize the watchdog
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    // initialize the clocks to ensure the timer works correctly
    hal::clocks::init_clocks_and_plls(
        XTAL_FREQ_HZ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    // initialize the ring oscillator for random number generation
    let mut rosc = hal::rosc::RingOscillator::new(pac.ROSC)
        .initialize();
    
    let timer = hal::Timer::new(pac.TIMER, &mut pac.RESETS);
    let mut previous_time = timer.get_counter();


    let mut chip8 = Chip8::new(rosc.next_u32());
    
    loop {
        let current_time = timer.get_counter();
        let elapsed_time = current_time
                .checked_duration_since(previous_time)
                .unwrap()
                .to_micros();
        chip8.update(elapsed_time as u32);
        previous_time = current_time;

        let mut delay_start = timer.get_counter_low();
        let mut delay = 15000;
        loop {
            let now = timer.get_counter_low();
            let waited = now.wrapping_sub(delay_start);
            if waited >= delay {
                break;
            }
            delay_start = now;
            delay -= waited;
        }
    }
}
