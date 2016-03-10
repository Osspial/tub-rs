use std::error::Error;
use std::fmt;

pub type TubResult<T> = Result<T, TubError>;
pub type GlCreationResult<T> = Result<T, GlCreationError>;

#[derive(Debug, Clone)]
pub enum TubError {
    OsError(String),
    IconLoadError(u16)
}

impl fmt::Display for TubError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::TubError::*;

        match *self {
            OsError(ref s) => write!(f, "{}", s),
            IconLoadError(size) => write!(f, "Could not load {0}x{0} icon", size)
        }
    }
}

impl Error for TubError {
    fn description<'a>(&'a self) -> &'a str {
        use self::TubError::*;

        match *self {
            OsError(ref s) => s,
            IconLoadError(_) => "Icon load error"
        }
    }
}

#[derive(Debug, Clone)]
pub enum GlCreationError {
    OsError(String, String),
    FloatingBufferError,
    SRGBBufferError,
    MSAABufferError,
    IndescribableFormatError(String),
    ExtendedCreationError,
    FunctionLoadError
}

impl fmt::Display for GlCreationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::GlCreationError::*;

        match *self {
            OsError(ref e, ref s)   => write!(f, "OS ERROR {}; Could not create OpenGl context; Reason: {}", e, s),
            FloatingBufferError     => write!(f, "Could not create floating-point color buffer"),
            SRGBBufferError         => write!(f, "Could not create SRGB pixel format"),
            MSAABufferError         => write!(f, "Could not create multisampled pixel format"),
            IndescribableFormatError(ref e) => 
                write!(f, "Indescribable Pixel Format; this likely occured because a pixel format \
                           was given that this hardware doesn't currently support. This can occur in \
                           a few situations: most notably, some intel drivers do not support multisampling \
                           and attempting to force that may be crashing the program. Another possible \
                           situation may be a lack of support for software opengl (or vice versa). The OS \
                           error string is as follows: {}", e),
            ExtendedCreationError   => write!(f, "Could not create OpenGl context with extended attributes"),
            FunctionLoadError       => write!(f, "Could not load functions for OpenGl context creation with attributes")
        }
    }
}

impl Error for GlCreationError {
    fn description(&self) -> &str {
        use self::GlCreationError::*;

        match *self {
            OsError(_, ref s)           => s,
            FloatingBufferError         => "Could not create floating-point color buffer",
            SRGBBufferError             => "Could not create SRGB pixel format",
            MSAABufferError             => "Could not create multisampled pixel format",
            IndescribableFormatError(_) => "Indescribable pixel format; see documentation for more details",
            ExtendedCreationError       => "Could not create OpenGl context with extended attributes",
            FunctionLoadError           => "Could not load functions for OpenGl context creation with attributes"
        }
    }
}