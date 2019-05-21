use crate::ffi;
use crate::renderer_services::RendererServices;
use crate::shading_system::{ShaderGroupRef, ShadingSystem};

use oiio::imagebuf::ImageBuf;
use oiio::imageio::ImageSpec;
use oiio::typedesc::TypeDesc;
use std::os::raw::{c_char, c_void};

use std::cell::RefCell;
use std::sync::Arc;

pub struct TestRenderer {
    pub rsw: ffi::RendererServicesWrapper,
    pub shaders: Vec<ShaderGroupRef>,
    width: i32,
    height: i32,
    output_vars: Vec<String>,
    pub output_bufs: Vec<ImageBuf>,
}

pub extern "C" fn renderer_services_supports(rs_obj: *const c_void, service: *const c_char) -> i32 {
    let service = unsafe {
        std::ffi::CStr::from_ptr(service)
            .to_string_lossy()
            .into_owned()
            .to_string()
    };

    let renderer: &TestRenderer = unsafe { std::mem::transmute(rs_obj) };

    renderer.supports(&service)
}

impl TestRenderer {
    pub unsafe fn new(width: i32, height: i32) -> Arc<RefCell<TestRenderer>> {
        let rsw = ffi::RendererServicesWrapper_create();
        let tr = Arc::new(RefCell::new(TestRenderer {
            rsw,
            shaders: Vec::new(),
            width,
            height,
            output_vars: Vec::new(),
            output_bufs: Vec::new(),
        }));
        ffi::RendererServicesWrapper_set_rust_object(
            rsw,
            &(*tr.borrow()) as *const TestRenderer as *const c_void,
        );
        ffi::RendererServicesWrapper_setfn_supports(rsw, renderer_services_supports);
        tr
    }

    pub fn supports(&self, service: &str) -> i32 {
        0
    }

    pub fn add_output(&mut self, varname: &str, filename: &str, td: TypeDesc, nchannels: i32) {
        let spec = ImageSpec::with_dimensions(self.width, self.height, nchannels, td);
        let buf = ImageBuf::create_with_spec(filename, spec).unwrap();
        oiio::imagebufalgo::zero(&buf);
        self.output_vars.push(varname.into());
        self.output_bufs.push(buf);
    }

    pub fn init_shading_system(&self, ss: &ShadingSystem) {}
    pub fn prepare_render(&self) {}
    pub fn warmup(&self) {}
    pub fn render(&self, xres: i32, yres: i32) {}
    pub fn clear(&self) {}
    pub fn finalize_pixel_buffer(&self) {}
}

impl RendererServices for TestRenderer {
    fn get_wrapper(&self) -> ffi::RendererServicesWrapper {
        self.rsw
    }
}
