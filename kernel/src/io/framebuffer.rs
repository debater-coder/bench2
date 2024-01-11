use bootloader_api::info::FrameBufferInfo;
use conquer_once::spin::OnceCell;
use spin::Mutex;

pub(crate) static FRAMEBUFFER: OnceCell<Framebuffer> = OnceCell::uninit();

pub(crate) struct Framebuffer {
    pub(crate) framebuffer_info: FrameBufferInfo,
    pub(crate) raw_framebuffer: Mutex<&'static mut [u8]>,
}

pub(crate) fn init(framebuffer: &'static mut bootloader_api::info::FrameBuffer) {
    FRAMEBUFFER.init_once(|| Framebuffer {
        framebuffer_info: framebuffer.info().clone(),
        raw_framebuffer: Mutex::new(framebuffer.buffer_mut()),
    });
}
