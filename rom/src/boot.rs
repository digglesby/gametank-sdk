use core::{arch::asm, panic::PanicInfo, ptr};

use crate::{main, sdk::scr::Console};

#[panic_handler]
fn panic(_panic: &PanicInfo<'_>) -> ! {
    loop {}
}

pub static mut VBLANK: bool = false;


unsafe extern "C" {
    #[inline(always)]
    pub unsafe fn null_interrupt();

    #[inline(always)]
    pub unsafe fn wait();

    #[inline(always)]
    pub unsafe fn enable_irq_handler();

    #[inline(always)]
    pub unsafe fn disable_irq_handler();

    pub unsafe static mut __rc50: u8;
    pub unsafe static mut __rc51: u8;
    pub unsafe static mut __rc0: u8;
    pub unsafe static mut __rc1: u8;


    unsafe static __data_load: u8;
    unsafe static mut __data_start: u8;
    unsafe static mut __data_end: u8;

    unsafe static __zp_load: u8;
    unsafe static mut __zp_start: u8;
    unsafe static mut __zp_end: u8;

    unsafe static mut __bss_start: u8;
    unsafe static mut __bss_end: u8;
}

#[inline(always)]
unsafe fn init_data_and_bss() {
    unsafe {
        // Copy .data from flash to RAM
        let mut src = &__data_load as *const u8;
        let mut dst = &raw mut __data_start as *mut u8;
        let end = &raw mut __data_end as *mut u8;
        while dst < end {
            dst.write_volatile(src.read_volatile());
            src = src.add(1);
            dst = dst.add(1);
        }

        // Zero .bss
        let mut bss = &raw mut __bss_start as *mut u8;
        let bss_end = &raw mut __bss_end as *mut u8;
        while bss < bss_end {
            bss.write_volatile(0);
            bss = bss.add(1);
        }

        // Copy .zp load to zp
        let mut src = &__zp_load as *const u8;
        let mut dst = &raw mut __zp_start as *mut u8;
        let end = &raw mut __zp_end as *mut u8;
        while dst < end {
            dst.write_volatile(src.read_volatile());
            src = src.add(1);
            dst = dst.add(1);
        }
    }
}

#[unsafe(no_mangle)]
extern "C" fn vblank_nmi() {
    unsafe { VBLANK = true; }
    unsafe { null_interrupt(); }
}


#[unsafe(link_section = ".vector_table")]
#[unsafe(no_mangle)]
pub static _VECTOR_TABLE: [unsafe extern "C" fn(); 3] = [
    vblank_nmi, // Non-Maskable Interrupt vector
    __boot, // Reset vector
    null_interrupt, // IRQ/BRK vector
];

#[unsafe(no_mangle)]
unsafe extern "C" fn __boot() {
    unsafe {
        reset_banking_register();
        init_stack();
        init_data_and_bss();

        main();
        core::panic!("Came out of main");
    }
}

#[inline(always)]
unsafe fn init_stack() {
    unsafe { __rc0 = 0xFF };
    unsafe { __rc1 = 0x1F }
}

#[inline(always)]
unsafe fn reset_banking_register() {
    let bank_reg: *mut u8 = 0x2005 as *mut u8;
    unsafe { ptr::write_volatile(bank_reg, 0) };
}
