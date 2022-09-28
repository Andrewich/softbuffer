#![doc = include_str!("../README.md")]

#[cfg(target_os = "macos")]
#[macro_use]
extern crate objc;
extern crate core;

#[cfg(target_os = "windows")]
mod win32;
#[cfg(target_os = "macos")]
mod cg;
#[cfg(target_os = "linux")]
mod x11;
#[cfg(target_os = "linux")]
mod wayland;
#[cfg(target_arch = "wasm32")]
mod web;

mod error;

pub use error::SoftBufferError;

use std::{
    sync::{Arc, Mutex},  
};

use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};

/// An instance of this struct contains the platform-specific data that must be managed in order to
/// write to a window on that platform. This struct owns the window that this data corresponds to
/// to ensure safety, as that data must be destroyed before the window itself is destroyed. You may
/// access the underlying window via [`window`](Self::window) and [`window_mut`](Self::window_mut).
pub struct GraphicsContext {
    buffer: Vec<u32>,  
    #[cfg(target_os = "windows")]
    graphics_context: Arc<Mutex<win32::Win32Context>>,
}

unsafe impl Send for GraphicsContext {}
unsafe impl Sync for GraphicsContext {}

impl GraphicsContext {
    /// Creates a new instance of this struct, consuming the given window.
    ///
    /// # Safety
    ///
    ///  - Ensure that the passed object is valid to draw a 2D buffer to
    pub unsafe fn new<W: HasRawWindowHandle>(window: &W) -> Result<Self, SoftBufferError> {
        let raw_window_handle = window.raw_window_handle();        
        
        let imple = match raw_window_handle {            
            #[cfg(target_os = "windows")]
            RawWindowHandle::Win32(win32_handle) => Arc::new(Mutex::new(win32::Win32Context::new::<W>(&win32_handle)?)),
            unimplemented_window_handle => return Err(SoftBufferError::UnsupportedPlatform {                
                human_readable_window_platform_name: window_handle_type_name(&unimplemented_window_handle),                
                window_handle: unimplemented_window_handle,                
            }),
        };
        
        let buffer = vec![0u32; 640 * 480];

        Ok(Self {
            buffer,
            graphics_context: imple,
        })
    }

    /// Gets shared access to the underlying window.
    // #[inline]
    // pub fn window(&self) -> &W {
    //     &self.window
    // }

    /// Gets mut/exclusive access to the underlying window.
    ///
    /// This method is `unsafe` because it could be used to replace the window with another one,
    /// thus dropping the original window and violating the property that this [`GraphicsContext`]
    /// will always be destroyed before the window it writes into. This method should only be used
    /// when the window type in use requires mutable access to perform some action on an existing
    /// window.
    ///
    /// # Safety
    ///
    /// - After the returned mutable reference is dropped, the window must still be the same window
    ///   which this [`GraphicsContext`] was created for; and within that window, the
    ///   platform-specific configuration for 2D drawing must not have been modified. (For example,
    ///   on macOS the view hierarchy of the window must not have been modified.)
    // #[inline]
    // pub unsafe fn window_mut(&mut self) -> &mut W {
    //     &mut self.window
    // }

    /// Shows the given buffer with the given width and height on the window corresponding to this
    /// graphics context. Panics if buffer.len() ≠ width*height. If the size of the buffer does
    /// not match the size of the window, the buffer is drawn in the upper-left corner of the window.
    /// It is recommended in most production use cases to have the buffer fill the entire window.
    /// Use your windowing library to find the size of the window.
    ///
    /// The format of the buffer is as follows. There is one u32 in the buffer for each pixel in
    /// the area to draw. The first entry is the upper-left most pixel. The second is one to the right
    /// etc. (Row-major top to bottom left to right one u32 per pixel). Within each u32 the highest
    /// order 8 bits are to be set to 0. The next highest order 8 bits are the red channel, then the
    /// green channel, and then the blue channel in the lowest-order 8 bits. See the examples for
    /// one way to build this format using bitwise operations.
    ///
    /// --------
    ///
    /// Pixel format (u32):
    ///
    /// 00000000RRRRRRRRGGGGGGGGBBBBBBBB
    ///
    /// 0: Bit is 0
    /// R: Red channel
    /// G: Green channel
    /// B: Blue channel
    #[inline]
    pub fn set_buffer(&mut self, width: u16, height: u16) {
        //if (width as usize) * (height as usize) != buffer.len() {
        //    panic!("The size of the passed buffer is not the correct size. Its length must be exactly width*height.");
        //}

        if let Ok(mut ctx) = self.graphics_context.lock() {
            unsafe {
                ctx.set_buffer(&self.buffer, width, height);
            }
        }        
    }
}

// impl<W: HasRawWindowHandle + HasRawDisplayHandle> AsRef<W> for GraphicsContext<W> {
//     /// Equivalent to [`self.window()`](Self::window()).
//     #[inline]
//     fn as_ref(&self) -> &W {
//         self.window()
//     }
// }

trait GraphicsContextImpl {
    unsafe fn set_buffer(&mut self, buffer: &[u32], width: u16, height: u16);
}

fn window_handle_type_name(handle: &RawWindowHandle) -> &'static str {
    match handle {
        RawWindowHandle::Xlib(_) => "Xlib",
        RawWindowHandle::Win32(_) => "Win32",
        RawWindowHandle::WinRt(_) => "WinRt",
        RawWindowHandle::Web(_) => "Web",
        RawWindowHandle::Wayland(_) => "Wayland",
        RawWindowHandle::AndroidNdk(_) => "AndroidNdk",
        RawWindowHandle::AppKit(_) => "AppKit",
        RawWindowHandle::Orbital(_) => "Orbital",
        RawWindowHandle::UiKit(_) => "UiKit",
        RawWindowHandle::Xcb(_) => "XCB",        
        RawWindowHandle::Haiku(_) => "Haiku",
        _ => "Unknown Name", //don't completely fail to compile if there is a new raw window handle type that's added at some point
    }
}
