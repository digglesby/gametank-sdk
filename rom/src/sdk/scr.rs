use bit_field::BitField;
use bitflags::Bits;
use bitflags::{self, Flags};
use volatile_register::WO;

use crate::boot::{_VECTOR_TABLE, disable_irq_handler, enable_irq_handler, wait};
use crate::sdk::scr;

bitflags::bitflags! {
    #[derive(Copy, Clone)]
    pub struct VideoFlags: u8 {
        const DMA_ENABLE           = 0b0000_0001;
        const DMA_PAGE_OUT        = 0b0000_0010;
        const DMA_NMI             = 0b0000_0100;
        const DMA_COLORFILL       = 0b0000_1000;
        const DMA_GCARRY          = 0b0001_0000;
        const DMA_CPU_TO_VRAM     = 0b0010_0000;
        const DMA_IRQ             = 0b0100_0000;
        const DMA_OPAQUE          = 0b1000_0000;
    }

    #[derive(Copy, Clone)]
    pub struct BankFlags: u8 {
        // Bits 0-2: Sprite RAM page (0–7)
        const SPRITE_PAGE_0       = 0b0000_0000;
        const SPRITE_PAGE_1       = 0b0000_0001;
        const SPRITE_PAGE_2       = 0b0000_0010;
        const SPRITE_PAGE_3       = 0b0000_0011;
        const SPRITE_PAGE_4       = 0b0000_0100;
        const SPRITE_PAGE_5       = 0b0000_0101;
        const SPRITE_PAGE_6       = 0b0000_0110;
        const SPRITE_PAGE_7       = 0b0000_0111;

        // Bit 3: Framebuffer select
        const FRAMEBUFFER_SELECT  = 0b0000_1000;

        // Bit 4: Clip L/R
        const CLIP_X              = 0b0001_0000;

        // Bit 5: Clip T/B
        const CLIP_Y              = 0b0010_0000;

        // Bits 6-7: RAM bank select
        const RAM_BANK_0          = 0b0000_0000;
        const RAM_BANK_1          = 0b0100_0000;
        const RAM_BANK_2          = 0b1000_0000;
        const RAM_BANK_3          = 0b1100_0000;
    }
}

pub enum VideoDma {
    DmaFb(Framebuffers),
    DmaBlit(Blitter),
    DmaSprites(SpriteMem),
}

impl VideoDma {
    #[inline(always)]
    fn framebuffers(self, sc: &mut SystemControl) -> Framebuffers {
        match self {
            VideoDma::DmaFb(framebuffers) => framebuffers,
            VideoDma::DmaBlit(blitter) => blitter.framebuffers(sc),
            VideoDma::DmaSprites(sprite_mem) => sprite_mem.framebuffers(sc),
        }
    }

    #[inline(always)]
    fn blitter(self, sc: &mut SystemControl) -> Blitter {
        match self {
            VideoDma::DmaFb(framebuffers) => framebuffers.blitter(sc),
            VideoDma::DmaBlit(blitter) => blitter,
            VideoDma::DmaSprites(sprite_mem) => sprite_mem.blitter(sc),
        }
    }

    #[inline(always)]
    fn sprite_mem(self, sc: &mut SystemControl) -> SpriteMem {
        match self {
            VideoDma::DmaFb(framebuffers) => framebuffers.sprite_mem(sc),
            VideoDma::DmaBlit(blitter) => blitter.sprite_mem(sc),
            VideoDma::DmaSprites(sprite_mem) => sprite_mem,
        }
    }
}

pub struct Framebuffers(());
pub struct Blitter(());
pub struct SpriteMem(());

// DMA_ENABLE == 0 -> CPU can see video memory
//   DMA_CPU_TO_VRAM == 1 -> Framebuffers
//   DMA_CPU_TO_VRAM == 0 -> Sprite RAM
// DMA_ENABLE == 1 -> Blitter Control Registers

impl Framebuffers {
    #[inline(always)]
    pub fn blitter(self, sc: &mut SystemControl) -> Blitter {
        sc.mir.video_reg.insert(VideoFlags::DMA_ENABLE);
        sc.scr.video_reg = sc.mir.video_reg;
        Blitter(())
    }

    #[inline(always)]
    pub fn sprite_mem(self, sc: &mut SystemControl) -> SpriteMem {
        // DMA_ENABLE is already false
        sc.mir.video_reg.remove(VideoFlags::DMA_CPU_TO_VRAM);
        sc.scr.video_reg = sc.mir.video_reg;
        SpriteMem(())
    }
}

impl SpriteMem {
    #[inline(always)]
    pub fn blitter(self, sc: &mut SystemControl) -> Blitter {
        sc.mir.video_reg.insert(VideoFlags::DMA_ENABLE);
        sc.scr.video_reg = sc.mir.video_reg;
        Blitter(())
    }

    #[inline(always)]
    pub fn framebuffers(self, sc: &mut SystemControl) -> Framebuffers {
        // DMA_ENABLE is already false
        sc.mir.video_reg.insert(VideoFlags::DMA_CPU_TO_VRAM);
        sc.scr.video_reg = sc.mir.video_reg;
        Framebuffers(())
    }
}

#[repr(C, packed)]
pub struct Bcr {
    pub fb_x: WO<u8>,
    pub fb_y: WO<u8>,
    pub vram_x: WO<u8>,
    pub vram_y: WO<u8>,
    pub width: WO<u8>,
    pub height: WO<u8>,
    pub start: WO<u8>,
    pub color: WO<u8>,
}

/// Blitter Control Registers
/// vram_VX 0x4000
/// vram_VY 0x4001
/// vram_GX 0x4002
/// vram_GY 0x4003
/// vram_WIDTH 0x4004
/// vram_HEIGHT 0x4005
/// vram_START 0x4006
/// vram_COLOR 0x4007
impl Bcr {
    #[inline(always)]
    unsafe fn new() -> &'static mut Bcr {
        unsafe { &mut *(0x4000 as *mut Bcr) }
    }
}

impl Blitter {
    #[inline(always)]
    pub fn framebuffers(self, sc: &mut SystemControl) -> Framebuffers {
        sc.mir.video_reg.remove(VideoFlags::DMA_ENABLE);
        sc.mir.video_reg.insert(VideoFlags::DMA_CPU_TO_VRAM);
        sc.scr.video_reg = sc.mir.video_reg;
        Framebuffers(())
    }

    #[inline(always)]
    pub fn sprite_mem(self, sc: &mut SystemControl) -> SpriteMem {
        sc.mir
            .video_reg
            .remove(VideoFlags::DMA_ENABLE | VideoFlags::DMA_CPU_TO_VRAM);
        sc.scr.video_reg = sc.mir.video_reg;
        SpriteMem(())
    }
}

/// System Control Register
/// $2000 	Write 1 to reset audio coprocessor
/// $2001 	Write 1 to send NMI to audio coprocessor
/// $2005 	Banking Register
/// $2006 	Audio enable and sample rate
/// $2007 	Video/Blitter Flags
#[repr(C, packed)]
pub struct Scr {
    pub audio_reset: u8,
    pub audio_nmi: u8,
    _pad0: [u8; 3], // Skips to $2005
    pub banking: BankFlags,
    pub audio_reg: u8,
    pub video_reg: VideoFlags,
}

#[used]
#[unsafe(link_section = ".data.zp")]
pub static mut SCR_MIR: Scr = Scr {
    audio_reset: 69,
    audio_nmi: 0,
    _pad0: [0; 3],
    banking: BankFlags::empty(),
    audio_reg: 0x69,
    video_reg: VideoFlags::empty(),
};

pub struct SystemControl {
    scr: &'static mut Scr,
    mir: &'static mut Scr,
}

impl SystemControl {
    pub fn init() -> Self {
        unsafe {
            // mir is zeroe'd
            let mir = &mut SCR_MIR;
            let scr = &mut *(0x2000 as *mut Scr);

            mir.video_reg.insert(VideoFlags::DMA_NMI);
            mir.video_reg.insert(VideoFlags::DMA_IRQ);
            mir.video_reg.insert(VideoFlags::DMA_GCARRY);
            mir.video_reg.insert(VideoFlags::DMA_OPAQUE);
            mir.video_reg.insert(VideoFlags::DMA_PAGE_OUT);
            mir.banking.remove(BankFlags::FRAMEBUFFER_SELECT);

            scr.audio_reset = mir.audio_reset;
            scr.audio_nmi = mir.audio_nmi;
            scr.banking = mir.banking;
            scr.audio_reg = mir.audio_reg;
            scr.video_reg = mir.video_reg;

            // clear_irq();

            Self { scr, mir }
        }
    }

    #[inline(always)]
    pub fn set_bank(&mut self, bank: u8) {
        unsafe {
            // disable_irq();
            _bank_shift_out(bank);
            // TODO: idk
            // enable_irq();
        }
    }

    //
    #[inline(always)]
    pub fn set_fill_mode(&mut self, mode: BlitterFillMode) {
        self.mir
            .video_reg
            .set(VideoFlags::DMA_COLORFILL, mode == BlitterFillMode::Color);
        self.scr.video_reg = self.mir.video_reg;
    }
}

#[derive(PartialEq)]
pub enum BlitterFillMode {
    Sprite,
    Color,
}

pub struct DmaManager {
    pub video_dma: Option<VideoDma>,
}

impl DmaManager {
    fn new(vdma: VideoDma) -> Self {
        Self {
            video_dma: Some(vdma),
        }
    }

    pub fn blitter(&mut self, sc: &mut SystemControl) -> Option<BlitterGuard> {
        let b = self.video_dma.take()?.blitter(sc);
        Some(BlitterGuard {
            dma_slot: &mut self.video_dma,
            inner: b,
        })
    }

    pub fn framebuffers(&mut self, sc: &mut SystemControl) -> Option<FramebuffersGuard> {
        let fb = self.video_dma.take()?.framebuffers(sc);
        Some(FramebuffersGuard {
            dma_slot: &mut self.video_dma,
            inner: fb,
        })
    }

    pub fn sprite_mem(&mut self, sc: &mut SystemControl) -> Option<SpriteMemGuard> {
        let sm = self.video_dma.take()?.sprite_mem(sc);
        Some(SpriteMemGuard {
            dma_slot: &mut self.video_dma,
            inner: sm,
        })
    }
}

pub struct Console {
    pub sc: SystemControl,
    pub dma: DmaManager,
}

pub struct BlitterGuard<'a> {
    dma_slot: &'a mut Option<VideoDma>,
    inner: Blitter,
}

impl<'a> Drop for BlitterGuard<'a> {
    fn drop(&mut self) {
        *self.dma_slot = Some(VideoDma::DmaBlit(Blitter(())));
    }
}

pub struct FramebuffersGuard<'a> {
    dma_slot: &'a mut Option<VideoDma>,
    inner: Framebuffers,
}

impl<'a> Drop for FramebuffersGuard<'a> {
    fn drop(&mut self) {
        *self.dma_slot = Some(VideoDma::DmaFb(Framebuffers(())));
    }
}

pub struct SpriteMemGuard<'a> {
    dma_slot: &'a mut Option<VideoDma>,
    inner: SpriteMem,
}

impl<'a> Drop for SpriteMemGuard<'a> {
    fn drop(&mut self) {
        *self.dma_slot = Some(VideoDma::DmaSprites(SpriteMem(())));
    }
}

impl<'a> SpriteMemGuard<'a> {
    #[inline(always)]
    pub fn bytes(&mut self) -> &mut [u8; 0x4000] {
        unsafe { &mut *(0x4000 as *mut [u8; 0x4000]) }
    }
}

impl<'a> FramebuffersGuard<'a> {
    #[inline(always)]
    pub fn bytes(&mut self) -> &mut [u8; 0x4000] {
        unsafe { &mut *(0x4000 as *mut [u8; 0x4000]) }
    }

    /// aliasing rules mean we can't borrow bytes and flip at the "same" time - I think?
    /// TODO: maybe flip returns a different framebufferguard, by consuming and returning?
    #[inline(always)]
    pub fn flip(&mut self, sc: &mut SystemControl) {
        unsafe {
            sc.mir.banking.toggle(BankFlags::FRAMEBUFFER_SELECT);
            sc.mir.video_reg.toggle(VideoFlags::DMA_PAGE_OUT);
            sc.scr.banking = sc.mir.banking;
            sc.scr.video_reg = sc.mir.video_reg;
        }
    }
}

impl<'a> BlitterGuard<'a> {
    #[inline(always)]
    pub fn draw_square(
        &mut self,
        sc: &mut SystemControl,
        x: u8,
        y: u8,
        width: u8,
        height: u8,
        color: u8,
    ) {
        sc.set_fill_mode(BlitterFillMode::Color);
        unsafe {
            let mut bcr = Bcr::new();
            bcr.fb_x.write(x);
            bcr.fb_y.write(y);
            bcr.width.write(width);
            bcr.height.write(height);
            bcr.color.write(color);
            bcr.start.write(1);
        }
    }

    #[inline(always)]
    pub fn draw_sprite(
        &mut self,
        sc: &mut SystemControl,
        sx: u8,
        sy: u8,
        fb_x: u8,
        fb_y: u8,
        width: u8,
        height: u8,
    ) {
        sc.set_fill_mode(BlitterFillMode::Sprite);
        unsafe {
            let mut bcr = Bcr::new();
            bcr.vram_x.write(sx);
            bcr.vram_y.write(sy);
            bcr.fb_x.write(fb_x);
            bcr.fb_y.write(fb_y);
            bcr.width.write(width);
            bcr.height.write(height);
            bcr.start.write(1);
        }
    }

    #[inline(always)]
    pub fn wait_blit(&self) {
        unsafe {
            wait();
            let mut bcr = Bcr::new();
            bcr.start.write(0);
        }
    }
}

impl Console {
    #[inline(always)]
    pub fn init() -> Self {
        // TODO: singleton-ize this?
        Self {
            sc: SystemControl::init(),
            dma: DmaManager::new(VideoDma::DmaSprites(SpriteMem(()))),
        }
    }
}

unsafe extern "C" {
    unsafe fn _bank_shift_out(bank: u8);
}

// impl Scr {
//     /// 0 = copy 16x16 across whole buffer
//     pub fn set_dma_gcarry(&mut self, gcarry: bool) {
//         unsafe {
//             VIDEO_REG = *VIDEO_REG.set_bit(4, gcarry);
//             (self.video_reg).write_volatile(VIDEO_REG);
//         }
//     }

//     pub fn set_vram_bank(&mut self, bank: u8) {
//         self.mirror.banking.update(|val| *val = *val.set_bits(0..3, bank));
//         self.scr.banking.write(self.mirror.banking.read());
//     }

//     #[link_section = ".text.fixed"]
//     pub unsafe fn new() -> MirroredScr {
//         let s = MirroredScr {
//             scr: &mut *(0x2000 as *mut Scr),
//             mirror: Scr {
//                 audio_reset: Volatile::new(0),
//                 audio_nmi: Volatile::new(0),
//                 _pad0: [0; 3], // Skips to $2005
//                 banking: Volatile::new(8),
//                 audio_reg: Volatile::new(0),
//                 video_reg: Volatile::new(69), // nice
//             }
//         };

//         s.scr.audio_reset.write(s.mirror.audio_reset.read());
//         s.scr.audio_nmi.write(s.mirror.audio_nmi.read());
//         s.scr.banking.write(s.mirror.banking.read());
//         s.scr.audio_reg.write(s.mirror.audio_reg.read());
//         s.scr.video_reg.write(s.mirror.video_reg.read());
//         s
//     }

//     pub fn flip_framebuffer(&mut self) {
//         self.mirror.video_reg.write(self.mirror.video_reg.read() ^ 0b00000010);
//         self.mirror.banking.write(self.mirror.banking.read() ^ 0b00001000);
//         self.scr.video_reg.write(self.mirror.video_reg.read());
//         self.scr.banking.write(self.mirror.banking.read());
//     }

//     pub fn set_colorfill_mode(&mut self, enable: bool) {
//         self.mirror.video_reg.update(|val| *val = *val.set_bit(3, enable));
//         self.scr.video_reg.write(self.mirror.video_reg.read());
//     }

//     pub fn enable_vblank_nmi(&mut self, enable: bool) {
//         self.mirror.video_reg.update(|val| *val = *val.set_bit(2, enable));
//         self.scr.video_reg.write(self.mirror.video_reg.read());
//     }

//     /// set dma enabled - 0 is DMA enabled, 1 allows access to blitter commands.
//     /// TODO correct, rename, whatever, do something to unfuck this mess.
//     pub fn set_dma_enable(&mut self, enable: bool) {
//         self .mirror.video_reg.update(|val| *val = *val.set_bit(0, enable));
//         self.scr.video_reg.write(self.mirror.video_reg.read());
//     }

//     pub fn set_dma_location(&mut self, location: DmaLocation) {
//         self.mirror.video_reg.update(|val| *val = *val.set_bit(5, location.value()));
//         self.scr.video_reg.write(self.mirror.video_reg.read());
//     }
// }
