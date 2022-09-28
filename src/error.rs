use std::error::Error;
use raw_window_handle::RawWindowHandle;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SoftBufferError {
    #[error(
        "The provided window returned an unsupported platform: {human_readable_window_platform_name}."
    )]
    UnsupportedPlatform {        
        human_readable_window_platform_name: &'static str,        
        window_handle: RawWindowHandle,        
    },
    #[error("Platform error")]
    PlatformError(Option<String>, Option<Box<dyn Error>>)
}

#[allow(unused)] // This isn't used on all platforms
pub(crate) fn unwrap<T, E: std::error::Error + 'static>(res: Result<T, E>, str: &str) -> Result<T, SoftBufferError>{
    match res{
        Ok(t) => Ok(t),
        Err(e) => Err(SoftBufferError::PlatformError(Some(str.into()), Some(Box::new(e))))
    }
}