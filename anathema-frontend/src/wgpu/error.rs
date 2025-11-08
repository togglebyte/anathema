use std::fmt::Display;

use winit::error::EventLoopError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    EventLoop(EventLoopError),
    CreateSurface(wgpu::CreateSurfaceError),
    Surface(wgpu::SurfaceError),
    Device(wgpu::RequestDeviceError),
    AdapterUnavailable,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EventLoop(err) => err.fmt(f),
            Self::CreateSurface(err) => err.fmt(f),
            Self::Surface(err) => err.fmt(f),
            Self::Device(err) => err.fmt(f),
            Self::AdapterUnavailable => write!(f, "Adapter unavailable"),
        }
    }
}

impl From<EventLoopError> for Error {
    fn from(err: EventLoopError) -> Self {
        Self::EventLoop(err)
    }
}

impl From<wgpu::CreateSurfaceError> for Error {
    fn from(err: wgpu::CreateSurfaceError) -> Self {
        Self::CreateSurface(err)
    }
}

impl From<wgpu::SurfaceError> for Error {
    fn from(err: wgpu::SurfaceError) -> Self {
        Self::Surface(err)
    }
}

impl From<wgpu::RequestDeviceError> for Error {
    fn from(err: wgpu::RequestDeviceError) -> Self {
        Self::Device(err)
    }
}
