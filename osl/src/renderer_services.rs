use crate::ffi;

pub trait RendererServices {
    fn get_wrapper(&self) -> ffi::RendererServicesWrapper;
}
