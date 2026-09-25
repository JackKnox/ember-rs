use crate::ffi;

use std::fmt;
use std::cell::UnsafeCell;

pub struct Allocator {
    pub sys: UnsafeCell<ffi::em_allocator>,
}

impl Allocator {
    pub fn system() -> Allocator {
        unsafe { Allocator { sys: UnsafeCell::new(ffi::emplat_system_allocator()) } }
    }
}

#[derive(Debug)]
pub enum Error {
    Timeout,
    Uninitilalized,
    InvalidEnum,
    InvalidValue,
    UnsupportedFormat,
    OutOfMemoryCPU,
    OutOfMemoryGPU,
    UnavailableAPI,
    Unimplemented,
    ValidationFailed,
    InUse,
    PermissionDenied,
    Unknown,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Timeout => write!(f, "Operation timed out before completion"),
            Error::Uninitilalized => {
                write!(f, "The system, device, or resource was not initialized")
            }
            Error::InvalidEnum => write!(f, "An invalid enum value was provided"),
            Error::InvalidValue => {
                write!(f, "An invalid value was provided (out of expected range)")
            }
            Error::UnsupportedFormat => write!(f, "The requested format or type is not supported"),
            Error::OutOfMemoryCPU => write!(f, "CPU memory allocation failed"),
            Error::OutOfMemoryGPU => write!(f, "GPU memory allocation failed"),
            Error::UnavailableAPI => write!(f, "The requested API is not available on this device"),
            Error::Unimplemented => {
                write!(f, "The requested feature or function is not implemented")
            }
            Error::ValidationFailed => write!(f, "Input or operation validation failed"),
            Error::InUse => write!(f, "The resource is currently in use and cannot be accessed"),
            Error::PermissionDenied => {
                write!(f, "The caller does not have the required permissions")
            }
            Error::Unknown => write!(
                f,
                "An unknown error has occurred; either the application has provided invalid input, or an implementation failure has occurred"
            ),
        }
    }
}

impl From<u32> for Error {
    fn from(_: u32) -> Self {
        todo!()
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Version(u32);

impl Version {
    pub const fn new(major: u32, minor: u32, patch: u32) -> Self {
        assert!(major <= 0x3FF, "major exceeds 10 bits");
        assert!(minor <= 0x3FF, "minor exceeds 10 bits");
        assert!(patch <= 0xFFF, "patch exceeds 12 bits");

        Self((major << 22) | (minor << 12) | patch)
    }

    pub const fn major(self) -> u32 {
        self.0 >> 22
    }

    pub const fn minor(self) -> u32 {
        (self.0 >> 12) & 0x3FF
    }

    pub const fn patch(self) -> u32 {
        self.0 & 0xFFF
    }

    pub const fn is_compliant(self, other: Version) -> bool {
        self.major() == other.major()
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major(), self.minor(), self.patch())
    }
}

impl From<u32> for Version {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<Version> for u32 {
    fn from(version: Version) -> u32 {
        version.0
    }
}
