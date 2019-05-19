use std::os::raw::{c_char, c_void};

use crate::ShaderGlobals;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ShadingSystem_api {
    _unused: [u8; 0],
}
pub type ShadingSystem = *mut ShadingSystem_api;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct RendererServices_api {
    _unused: [u8; 0],
}
pub type RendererServices = *mut RendererServices_api;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct RendererServicesWrapper_api {
    _unused: [u8; 0],
}
pub type RendererServicesWrapper = *mut RendererServicesWrapper_api;

type FnRswSupports = extern "C" fn(*const c_void, *const c_char) -> i32;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ErrorHandler_api {
    _unused: [u8; 0],
}
pub type ErrorHandler = *mut ErrorHandler_api;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ShaderGroupRef_api {
    _unused: [u8; 0],
}
pub type ShaderGroupRef = *mut ShaderGroupRef_api;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PerThreadInfo_api {
    _unused: [u8; 0],
}
pub type PerThreadInfo = *mut PerThreadInfo_api;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ShadingContext_api {
    _unused: [u8; 0],
}
pub type ShadingContext = *mut ShadingContext_api;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ClosureColor_api {
    _unused: [u8; 0],
}
pub type ClosureColorPtr = *mut ClosureColor_api;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ShaderSymbol_api {
    _unused: [u8; 0],
}
pub type ShaderSymbolPtr = *mut ShaderSymbol_api;

#[repr(i32)]
pub enum VerbosityLevel {
    Quiet = 0,
    Normal = 1,
    Verbose = 2,
}

#[repr(i32)]
pub enum ErrCode {
    Message = 0 << 16,
    Info = 1 << 16,
    Warning = 2 << 16,
    Error = 3 << 16,
    Severe = 4 << 16,
    Debug = 5 << 16,
}

#[repr(C)]
pub(crate) struct ClosureParam {
    pub typedesc: oiio::typedesc::TypeDesc,
    pub offset: i32,
    pub key: *const c_char,
    pub field_size: i32,
}

#[link(name = "osl_capi", kind = "static")]
extern "C" {
    pub(crate) fn ShadingSystem_create(renderer: RendererServicesWrapper) -> ShadingSystem;
    pub(crate) fn ShadingSystem_create_with_error_handler(
        renderer: RendererServicesWrapper,
        eh: ErrorHandler,
    ) -> ShadingSystem;
    pub(crate) fn ShadingSystem_destroy(ss: ShadingSystem);
    pub(crate) fn ShadingSystem_register_closure(
        ss: ShadingSystem,
        name: *const c_char,
        id: i32,
        params: *const ClosureParam,
    );
    pub(crate) fn ShadingSystem_attribute(
        ss: ShadingSystem,
        name: *const c_char,
        typedesc: oiio::typedesc::TypeDesc,
        val: *const c_void,
    ) -> bool;
    pub(crate) fn ShadingSystem_group_attribute(
        ss: ShadingSystem,
        group: ShaderGroupRef,
        name: *const c_char,
        typedesc: oiio::typedesc::TypeDesc,
        val: *const c_void,
    ) -> bool;
    pub(crate) fn ShadingSystem_shader_group_begin(
        ss: ShadingSystem,
        group_name: *const c_char,
    ) -> ShaderGroupRef;
    pub(crate) fn ShadingSystem_shader_group_end(ss: ShadingSystem, group: ShaderGroupRef);
    pub(crate) fn ShadingSystem_shader(
        ss: ShadingSystem,
        group: ShaderGroupRef,
        shaderusage: *const c_char,
        shadername: *const c_char,
        layername: *const c_char,
    ) -> bool;
    pub(crate) fn ShadingSystem_create_thread_info(ss: ShadingSystem) -> PerThreadInfo;
    pub(crate) fn ShadingSystem_destroy_thread_info(ss: ShadingSystem, tinfo: PerThreadInfo);
    pub(crate) fn ShadingSystem_get_context(
        ss: ShadingSystem,
        tinfo: PerThreadInfo,
    ) -> ShadingContext;
    pub(crate) fn ShadingSystem_release_context(ss: ShadingSystem, context: ShadingContext);

    pub(crate) fn ShadingSystem_execute(
        ss: ShadingSystem,
        context: ShadingContext,
        group: ShaderGroupRef,
        sg: *const ShaderGlobals,
        run: bool,
    ) -> bool;
    pub(crate) fn ShadingSystem_find_symbol(
        ss: ShadingSystem,
        group: ShaderGroupRef,
        symbolname: *const c_char,
    ) -> ShaderSymbolPtr;
    pub(crate) fn ShadingSystem_symbol_typedesc(
        ss: ShadingSystem,
        symbol: ShaderSymbolPtr,
    ) -> oiio::typedesc::TypeDesc;
    pub(crate) fn ShadingSystem_symbol_address(
        ss: ShadingSystem,
        ctx: ShadingContext,
        symbol: ShaderSymbolPtr,
    ) -> *const c_void;

    pub(crate) fn ShaderGroup_destroy(group: ShaderGroupRef);

    pub(crate) fn RendererServices_create() -> RendererServices;
    pub(crate) fn RendererServices_destroy(rs: RendererServices);

    pub(crate) fn RendererServicesWrapper_create() -> RendererServicesWrapper;
    pub(crate) fn RendererServicesWrapper_destroy(rsw: RendererServicesWrapper);
    pub(crate) fn RendererServicesWrapper_set_rust_object(
        rsw: RendererServicesWrapper,
        rs_obj: *const c_void,
    );
    pub(crate) fn RendererServicesWrapper_setfn_supports(
        rsw: RendererServicesWrapper,
        supports: FnRswSupports,
    );

    pub(crate) fn ErrorHandler_create(
        error_handler: extern "C" fn(i32, *const c_char),
    ) -> ErrorHandler;
    pub(crate) fn ErrorHandler_destroy(eh: ErrorHandler);
    pub(crate) fn ErrorHandler_set_verbosity(eh: ErrorHandler, verbosity: i32);
    pub(crate) fn ErrorHandler_get_verbosity(eh: ErrorHandler) -> i32;

    pub(crate) fn shade_image(
        ss: ShadingSystem,
        group: ShaderGroupRef,
        sg: *const ShaderGlobals,
        imagebuf: oiio::ffi::ImageBuf,
        outputs: *const oiio::Ustring,
        noutputs: i32,
        shadelocations: i32,
        roi: oiio::imageio::ROI,
    ) -> bool;
}
