use crate::graphics::framebuffer_images::FramebufferImages;

// Supported: RGBA1555
pub type PixelType = crate::graphics::color::RGBA5551;

pub const TV_TYPE_PAL: u8 = 0;
pub const TV_TYPE_NTSC: u8 = 1;
pub const TV_TYPE_MPAL: u8 = 2;

const WIDTH: u32 = 320;
const HEIGHT: u32 = 240;

const FRAMEBUFFER_ALIGNMENT: usize = 32;

const VI_BASE_REG: *mut u32 = 0xA440_0000 as *mut u32;

pub struct Video {
    framebuffers: FramebufferImages<PixelType>,
}

#[allow(dead_code)]
enum RegisterOffset {
    Status = 0x00,
    DRAMAddress = 0x04,
    HWidth = 0x08,
    VIntr = 0x0C,
    Current = 0x10,
    Timing = 0x14,
    VSync = 0x18,
    HSync = 0x1C,
    HSyncLeap = 0x20,
    HVideo = 0x24,
    VVideo = 0x28,
    VBurst = 0x2C,
    XScale = 0x30,
    YScale = 0x34,
}

impl Video {
    pub const fn new() -> Self {
        Self {
            framebuffers: FramebufferImages::new(),
        }
    }

    pub fn init(&self, tv_type: u8) {
        unsafe {
            VI_BASE_REG.add(RegisterOffset::Status as usize >> 2).write_volatile(12878);
            VI_BASE_REG.add(RegisterOffset::VIntr as usize >> 2).write_volatile(2);
            match tv_type {
                TV_TYPE_PAL => {
                    VI_BASE_REG.add(RegisterOffset::Timing as usize >> 2).write_volatile(0x0404_233A);
                    VI_BASE_REG.add(RegisterOffset::VSync as usize >> 2).write_volatile(0x0000_0271);
                    VI_BASE_REG.add(RegisterOffset::HSync as usize >> 2).write_volatile(0x0015_0C69);
                    VI_BASE_REG.add(RegisterOffset::HSyncLeap as usize >> 2).write_volatile(0xC6F0_C6E);
                    VI_BASE_REG.add(RegisterOffset::HVideo as usize >> 2).write_volatile(0x0080_0300);
                    VI_BASE_REG.add(RegisterOffset::VVideo as usize >> 2).write_volatile(0x002D_026D);
                    VI_BASE_REG.add(RegisterOffset::VBurst as usize >> 2).write_volatile(0x0009_026B);
                    VI_BASE_REG.add(RegisterOffset::XScale as usize >> 2).write_volatile(512);
                    VI_BASE_REG.add(RegisterOffset::YScale as usize >> 2).write_volatile(853);
                }
                TV_TYPE_MPAL => {
                    VI_BASE_REG.add(RegisterOffset::Timing as usize >> 2).write_volatile(0x0465_1E39);
                    VI_BASE_REG.add(RegisterOffset::VSync as usize >> 2).write_volatile(0x0000_020D);
                    VI_BASE_REG.add(RegisterOffset::HSync as usize >> 2).write_volatile(0x0000_0C10);
                    VI_BASE_REG.add(RegisterOffset::HSyncLeap as usize >> 2).write_volatile(0x0C1C_0C1C);
                    VI_BASE_REG.add(RegisterOffset::HVideo as usize >> 2).write_volatile(0x006C_02EC);
                    VI_BASE_REG.add(RegisterOffset::VVideo as usize >> 2).write_volatile(0x0023_0203);
                    VI_BASE_REG.add(RegisterOffset::VBurst as usize >> 2).write_volatile(0x000E_0204);
                    VI_BASE_REG.add(RegisterOffset::XScale as usize >> 2).write_volatile(512);
                    VI_BASE_REG.add(RegisterOffset::YScale as usize >> 2).write_volatile(1024);
                }
                TV_TYPE_NTSC | _ => {
                    VI_BASE_REG.add(RegisterOffset::Timing as usize >> 2).write_volatile(0x03E5_2239);
                    VI_BASE_REG.add(RegisterOffset::VSync as usize >> 2).write_volatile(0x0000_020D);
                    VI_BASE_REG.add(RegisterOffset::HSync as usize >> 2).write_volatile(0x0000_0C15);
                    VI_BASE_REG.add(RegisterOffset::HSyncLeap as usize >> 2).write_volatile(0x0C15_0C15);
                    VI_BASE_REG.add(RegisterOffset::HVideo as usize >> 2).write_volatile(0x006C_02EC);
                    VI_BASE_REG.add(RegisterOffset::VVideo as usize >> 2).write_volatile(0x0025_01FF);
                    VI_BASE_REG.add(RegisterOffset::VBurst as usize >> 2).write_volatile(0x000E_0204);
                    VI_BASE_REG.add(RegisterOffset::XScale as usize >> 2).write_volatile(512);
                    VI_BASE_REG.add(RegisterOffset::YScale as usize >> 2).write_volatile(1024);
                }
            }
        }
    }

    pub fn framebuffers(&self) -> &FramebufferImages<PixelType> { &self.framebuffers }

    pub fn alloc_framebuffer(&self) {
        self.framebuffers.alloc_buffers(FRAMEBUFFER_ALIGNMENT, WIDTH, HEIGHT);
        self.activate_frontbuffer();
        unsafe { VI_BASE_REG.add(RegisterOffset::HWidth as usize >> 2).write_volatile(WIDTH); }
    }

    pub fn swap_buffers(&self) {
        self.framebuffers.swap_buffers();
        self.activate_frontbuffer();
    }

    fn activate_frontbuffer(&self) {
        let mut frontbuffer_lock = self.framebuffers.frontbuffer().lock();
        if let Some(frontbuffer) = frontbuffer_lock.as_mut() {
            let pixels = frontbuffer.pixels_mut();
            let ptr = pixels.as_ptr();
            let dram_address = ((ptr as u32) & 0x1FFF_FFFF) | 0xA000_0000;

            // The framebuffer is accessed cached by the CPU, so invalidate it now
            unsafe {
                crate::cop0::dcache_index_writeback_invalidate_range(
                    ptr as usize,
                    core::mem::size_of_val(pixels),
                )
            };

            unsafe {
                VI_BASE_REG.add(RegisterOffset::DRAMAddress as usize >> 2).write_volatile(dram_address);
            }
        }
    }
}
