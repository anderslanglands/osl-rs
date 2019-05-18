mod ffi;
use ffi::{ErrCode, PerThreadInfo, RendererServicesWrapper, ShadingContext, VerbosityLevel};
mod math;
pub use math::*;

use std::os::raw::{c_char, c_void};

use std::cell::RefCell;
use std::sync::Arc;

use oiio::imagebuf::ImageBuf;
use oiio::imageio::{ImageSpec, ROI};
use oiio::typedesc;
use oiio::typedesc::TypeDesc;
use oiio::Ustring;

pub struct ClosureParam {
    pub typedesc: TypeDesc,
    pub offset: usize,
    pub key: Option<String>,
    pub field_size: usize,
}

impl From<&ClosureParam> for ffi::ClosureParam {
    fn from(rcp: &ClosureParam) -> ffi::ClosureParam {
        let key = match &rcp.key {
            // NOTE: We are leaking memory here, but as long as we don't
            // create new closures in an inner loop that shouldn't be an issue
            Some(s) => std::ffi::CString::new(s.as_str()).unwrap().into_raw(),
            None => std::ptr::null(),
        };

        ffi::ClosureParam {
            typedesc: rcp.typedesc,
            offset: rcp.offset as i32,
            key,
            field_size: rcp.field_size as i32,
        }
    }
}

pub struct ClosureDef {
    name: String,
    id: i32,
    params: Vec<ClosureParam>,
}

pub struct TestRenderer {
    pub rsw: ffi::RendererServicesWrapper,
    shaders: Vec<ShaderGroupRef>,
    width: i32,
    height: i32,
    output_vars: Vec<String>,
    output_bufs: Vec<ImageBuf>,
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

pub trait RendererServices {
    fn get_wrapper(&self) -> ffi::RendererServicesWrapper;
}

impl RendererServices for TestRenderer {
    fn get_wrapper(&self) -> ffi::RendererServicesWrapper {
        self.rsw
    }
}

pub trait ShadingSystemAttribute {
    const typedesc: TypeDesc;

    fn set_attribute(&self, name: &str, ss: ffi::ShadingSystem) -> bool;
    fn set_group_attribute(
        &self,
        name: &str,
        ss: ffi::ShadingSystem,
        group: &ShaderGroupRef,
    ) -> bool;
}

impl ShadingSystemAttribute for i32 {
    const typedesc: TypeDesc = typedesc::INT32;

    fn set_attribute(&self, name: &str, ss: ffi::ShadingSystem) -> bool {
        let name = std::ffi::CString::new(name).unwrap();
        unsafe {
            ffi::ShadingSystem_attribute(
                ss,
                name.as_ptr(),
                Self::typedesc,
                self as *const i32 as *const c_void,
            )
        }
    }

    fn set_group_attribute(
        &self,
        name: &str,
        ss: ffi::ShadingSystem,
        group: &ShaderGroupRef,
    ) -> bool {
        let name = std::ffi::CString::new(name).unwrap();
        unsafe {
            ffi::ShadingSystem_group_attribute(
                ss,
                group.group,
                name.as_ptr(),
                Self::typedesc,
                self as *const i32 as *const c_void,
            )
        }
    }
}

impl ShadingSystemAttribute for &[i32] {
    const typedesc: TypeDesc = typedesc::INT32;

    fn set_attribute(&self, name: &str, ss: ffi::ShadingSystem) -> bool {
        let name = std::ffi::CString::new(name).unwrap();
        unsafe {
            ffi::ShadingSystem_attribute(
                ss,
                name.as_ptr(),
                TypeDesc::new(
                    Self::typedesc.basetype,
                    Self::typedesc.aggregate,
                    Self::typedesc.vecsemantics,
                    self.len() as i32,
                ),
                self.as_ptr() as *const i32 as *const c_void,
            )
        }
    }

    fn set_group_attribute(
        &self,
        name: &str,
        ss: ffi::ShadingSystem,
        group: &ShaderGroupRef,
    ) -> bool {
        let name = std::ffi::CString::new(name).unwrap();
        unsafe {
            ffi::ShadingSystem_group_attribute(
                ss,
                group.group,
                name.as_ptr(),
                TypeDesc::new(
                    Self::typedesc.basetype,
                    Self::typedesc.aggregate,
                    Self::typedesc.vecsemantics,
                    self.len() as i32,
                ),
                self.as_ptr() as *const i32 as *const c_void,
            )
        }
    }
}

impl ShadingSystemAttribute for f32 {
    const typedesc: TypeDesc = typedesc::FLOAT;

    fn set_attribute(&self, name: &str, ss: ffi::ShadingSystem) -> bool {
        let name = std::ffi::CString::new(name).unwrap();
        unsafe {
            ffi::ShadingSystem_attribute(
                ss,
                name.as_ptr(),
                Self::typedesc,
                self as *const f32 as *const c_void,
            )
        }
    }

    fn set_group_attribute(
        &self,
        name: &str,
        ss: ffi::ShadingSystem,
        group: &ShaderGroupRef,
    ) -> bool {
        let name = std::ffi::CString::new(name).unwrap();
        unsafe {
            ffi::ShadingSystem_group_attribute(
                ss,
                group.group,
                name.as_ptr(),
                Self::typedesc,
                self as *const f32 as *const c_void,
            )
        }
    }
}

impl ShadingSystemAttribute for &[f32] {
    const typedesc: TypeDesc = typedesc::FLOAT;

    fn set_attribute(&self, name: &str, ss: ffi::ShadingSystem) -> bool {
        let name = std::ffi::CString::new(name).unwrap();
        unsafe {
            ffi::ShadingSystem_attribute(
                ss,
                name.as_ptr(),
                TypeDesc::new(
                    Self::typedesc.basetype,
                    Self::typedesc.aggregate,
                    Self::typedesc.vecsemantics,
                    self.len() as i32,
                ),
                self.as_ptr() as *const f32 as *const c_void,
            )
        }
    }

    fn set_group_attribute(
        &self,
        name: &str,
        ss: ffi::ShadingSystem,
        group: &ShaderGroupRef,
    ) -> bool {
        let name = std::ffi::CString::new(name).unwrap();
        unsafe {
            ffi::ShadingSystem_group_attribute(
                ss,
                group.group,
                name.as_ptr(),
                TypeDesc::new(
                    Self::typedesc.basetype,
                    Self::typedesc.aggregate,
                    Self::typedesc.vecsemantics,
                    self.len() as i32,
                ),
                self.as_ptr() as *const f32 as *const c_void,
            )
        }
    }
}

impl ShadingSystemAttribute for &str {
    const typedesc: TypeDesc = typedesc::STRING;

    fn set_attribute(&self, name: &str, ss: ffi::ShadingSystem) -> bool {
        let name = std::ffi::CString::new(name).unwrap();
        let value = std::ffi::CString::new(*self).unwrap();
        unsafe {
            ffi::ShadingSystem_attribute(
                ss,
                name.as_ptr(),
                Self::typedesc,
                value.as_ptr() as *const c_void,
            )
        }
    }

    fn set_group_attribute(
        &self,
        name: &str,
        ss: ffi::ShadingSystem,
        group: &ShaderGroupRef,
    ) -> bool {
        let name = std::ffi::CString::new(name).unwrap();
        let value = std::ffi::CString::new(*self).unwrap();
        unsafe {
            ffi::ShadingSystem_group_attribute(
                ss,
                group.group,
                name.as_ptr(),
                Self::typedesc,
                value.as_ptr() as *const c_void,
            )
        }
    }
}

impl ShadingSystemAttribute for &[String] {
    const typedesc: TypeDesc = typedesc::STRING;

    fn set_attribute(&self, name: &str, ss: ffi::ShadingSystem) -> bool {
        let name = std::ffi::CString::new(name).unwrap();
        let values = self
            .iter()
            .map(|v| std::ffi::CString::new(v.as_str()).unwrap())
            .collect::<Vec<_>>();
        let value_ptrs = values.iter().map(|v| v.as_ptr()).collect::<Vec<_>>();

        unsafe {
            ffi::ShadingSystem_attribute(
                ss,
                name.as_ptr(),
                TypeDesc::new(
                    Self::typedesc.basetype,
                    Self::typedesc.aggregate,
                    Self::typedesc.vecsemantics,
                    self.len() as i32,
                ),
                value_ptrs.as_ptr() as *const c_void,
            )
        }
    }

    fn set_group_attribute(
        &self,
        name: &str,
        ss: ffi::ShadingSystem,
        group: &ShaderGroupRef,
    ) -> bool {
        let name = std::ffi::CString::new(name).unwrap();
        let values = self
            .iter()
            .map(|v| std::ffi::CString::new(v.as_str()).unwrap())
            .collect::<Vec<_>>();
        let value_ptrs = values.iter().map(|v| v.as_ptr()).collect::<Vec<_>>();

        unsafe {
            ffi::ShadingSystem_group_attribute(
                ss,
                group.group,
                name.as_ptr(),
                TypeDesc::new(
                    Self::typedesc.basetype,
                    Self::typedesc.aggregate,
                    Self::typedesc.vecsemantics,
                    self.len() as i32,
                ),
                value_ptrs.as_ptr() as *const c_void,
            )
        }
    }
}

impl ShadingSystemAttribute for &[&str] {
    const typedesc: TypeDesc = typedesc::STRING;

    fn set_attribute(&self, name: &str, ss: ffi::ShadingSystem) -> bool {
        let name = std::ffi::CString::new(name).unwrap();
        let values = self
            .iter()
            .map(|v| std::ffi::CString::new(*v).unwrap())
            .collect::<Vec<_>>();
        let value_ptrs = values.iter().map(|v| v.as_ptr()).collect::<Vec<_>>();

        unsafe {
            ffi::ShadingSystem_attribute(
                ss,
                name.as_ptr(),
                TypeDesc::new(
                    Self::typedesc.basetype,
                    Self::typedesc.aggregate,
                    Self::typedesc.vecsemantics,
                    self.len() as i32,
                ),
                value_ptrs.as_ptr() as *const c_void,
            )
        }
    }

    fn set_group_attribute(
        &self,
        name: &str,
        ss: ffi::ShadingSystem,
        group: &ShaderGroupRef,
    ) -> bool {
        let name = std::ffi::CString::new(name).unwrap();
        let values = self
            .iter()
            .map(|v| std::ffi::CString::new(*v).unwrap())
            .collect::<Vec<_>>();
        let value_ptrs = values.iter().map(|v| v.as_ptr()).collect::<Vec<_>>();

        unsafe {
            ffi::ShadingSystem_group_attribute(
                ss,
                group.group,
                name.as_ptr(),
                TypeDesc::new(
                    Self::typedesc.basetype,
                    Self::typedesc.aggregate,
                    Self::typedesc.vecsemantics,
                    self.len() as i32,
                ),
                value_ptrs.as_ptr() as *const c_void,
            )
        }
    }
}

pub struct ShadingSystem {
    ss: ffi::ShadingSystem,
    renderer: Arc<RefCell<dyn RendererServices>>,
    error_handler: ffi::ErrorHandler,
}

impl ShadingSystem {
    pub fn new(renderer: Arc<RefCell<dyn RendererServices>>) -> ShadingSystem {
        let error_handler = unsafe { ffi::ErrorHandler_create(handle_errors) };

        let ss = unsafe {
            ffi::ShadingSystem_create_with_error_handler(
                renderer.borrow().get_wrapper(),
                error_handler,
            )
        };

        ShadingSystem {
            ss,
            renderer,
            error_handler,
        }
    }

    pub fn register_closure(&mut self, name: &str, id: i32, params: &[ClosureParam]) {
        let name = std::ffi::CString::new(name).unwrap();

        let closure_params = params
            .iter()
            .map(|p| ffi::ClosureParam::from(p))
            .collect::<Vec<_>>();

        unsafe {
            ffi::ShadingSystem_register_closure(
                self.ss,
                name.as_ptr(),
                id,
                closure_params.as_ptr(),
            );
        }
    }

    /// Set an attribute controlling the shading system.  Return true
    /// if the name and type were recognized and the attrib was set.
    /// Documented attributes are as follows:
    /// 1. Attributes that should be exposed to users:
    ///    int statistics:level   Automatically print OSL statistics (0).
    ///    string searchpath:shader  Colon-separated path to search for .oso
    ///                                files ("", meaning test "." only)
    ///    string colorspace      Name of RGB color space ("Rec709")
    ///    int range_checking     Generate extra code for component & array
    ///                              range checking (1)
    ///    int debug_nan          Add extra (expensive) code to pinpoint
    ///                              when NaN/Inf happens (0).
    ///    int debug_uninit       Add extra (expensive) code to pinpoint
    ///                              use of uninitialized variables (0).
    ///    int compile_report     Issue info messages to the renderer for
    ///                              every shader compiled (0).
    ///    int max_warnings_per_thread  Number of warning calls that should be
    ///                              processed per thread (100).
    ///    int buffer_printf      Buffer printf output from shaders and
    ///                              output atomically, to prevent threads
    ///                              from interleaving lines. (1)
    ///    int profile            Perform some rudimentary profiling (0)
    ///    int no_noise           Replace noise with constant value. (0)
    ///    int no_pointcloud      Skip pointcloud lookups. (0)
    ///    int exec_repeat        How many times to run each group (1).
    ///    int opt_warnings       Warn on certain failure to runtime-optimize
    ///                              cetain shader constructs. (0)
    ///    int gpu_opt_error      Consider a hard error if certain shader
    ///                              constructs cannot be optimized away. (0)
    /// 2. Attributes that should be set by applications/renderers that
    /// incorporate OSL:
    ///    string commonspace     Name of "common" coord system ("world")
    ///    string[] raytypes      Array of ray type names
    ///    string[] renderer_outputs
    ///                           Array of names of renderer outputs (AOVs)
    ///                              that should not be optimized away.
    ///    int unknown_coordsys_error  Should errors be issued when unknown
    ///                              coord system names are used? (1)
    ///    int connection_error   Should errors be issued when ConnectShaders
    ///                              fails to find the layer or parameter? (1)
    ///    int strict_messages    Issue error if a message is set after
    ///                              being queried (1).
    ///    int error_repeats      If zero, suppress repeats of errors and
    ///                              warnings that are exact duplicates of
    ///                              earlier ones. (1)
    ///    int lazylayers         Evaluate shader layers only when their
    ///                              outputs are first needed (1)
    ///    int lazyglobals        Run layers lazily even if they write to
    ///                              globals (1)
    ///    int lazyunconnected    Run layers lazily even if they have no
    ///                              output connections (1). For debugging.
    ///    int lazy_userdata      Retrieve userdata lazily (0).
    ///    int userdata_isconnected  Should lockgeom=0 params (that may
    ///                              receive userdata) return true from
    ///                              isconnected()? (0)
    ///    int greedyjit          Optimize and compile all shaders up front,
    ///                              versus only as needed (0).
    ///    int lockgeom           Default 'lockgeom' value for shader params
    ///                              that don't specify it (1).  Lockgeom
    ///                              means a param CANNOT be overridden by
    ///                              interpolated geometric parameters.
    ///    int countlayerexecs    Add extra code to count total layers run.
    ///    int allow_shader_replacement Allow shader to be specified more than
    ///                              once, replacing former definition.
    ///    string archive_groupname  Name of a group to pickle and archive.
    ///    string archive_filename   Name of file to save the group archive.
    /// 3. Attributes that that are intended for developers debugging
    /// liboslexec itself:
    /// These attributes may be helpful for liboslexec developers or
    /// for debugging, but probably not for using OSL in production:
    ///    int debug              Set debug output level (0)
    ///    int clearmemory        Zero out working memory before each shade (0)
    ///    int optimize           Runtime optimization level (2)
    ///       And there are several int options that, if set to 0, will turn
    ///       off individual classes of runtime optimizations:
    ///         opt_simplify_param, opt_constant_fold, opt_stale_assign,
    ///         opt_elide_useless_ops, opt_elide_unconnected_outputs,
    ///         opt_peephole, opt_coalesce_temps, opt_assign, opt_mix
    ///         opt_merge_instances, opt_merge_instance_with_userdata,
    ///         opt_fold_getattribute, opt_middleman, opt_texture_handle
    ///         opt_seed_bblock_aliases
    ///    int opt_passes         Number of optimization passes per layer (10)
    ///    int llvm_optimize      Which of several LLVM optimize strategies (0)
    ///    int llvm_debug         Set LLVM extra debug level (0)
    ///    int llvm_debug_layers  Extra printfs upon entering and leaving
    ///                              layer functions.
    ///    int llvm_debug_ops     Extra printfs for each OSL op (helpful
    ///                              for devs to find crashes)
    ///    int llvm_output_bitcode  Output the full bitcode for each group,
    ///                              for debugging. (0)
    ///    int max_local_mem_KB   Error if shader group needs more than this
    ///                              much local storage to execute (1024K)
    ///    string debug_groupname Name of shader group -- debug only this one
    ///    string debug_layername Name of shader layer -- debug only this one
    ///    int optimize_nondebug  If 1, fully optimize shaders that are not
    ///                              designated as the debug shaders.
    ///    string opt_layername   If set, only optimize the named layer
    ///    string only_groupname  Compile only this one group (skip all others)
    ///    int force_derivs       Force all float-based variables to compute
    ///                              and store derivatives. (0)
    ///
    /// Note: the attributes referred to as "string" are actually on the app
    /// side as ustring or const char* (they have the same data layout), NOT
    /// std::string!
    pub fn attribute<T: ShadingSystemAttribute>(&mut self, name: &str, val: T) -> bool {
        val.set_attribute(name, self.ss)
    }

    pub fn group_attribute<T: ShadingSystemAttribute>(
        &mut self,
        group: &ShaderGroupRef,
        name: &str,
        val: T,
    ) -> bool {
        val.set_group_attribute(name, self.ss, group)
    }

    pub fn shader_group_begin(&self, group_name: &str) -> ShaderGroupRef {
        let group_name = std::ffi::CString::new(group_name).unwrap();
        unsafe {
            Arc::new(ShaderGroup {
                group: ffi::ShadingSystem_shader_group_begin(self.ss, group_name.as_ptr()),
            })
        }
    }

    pub fn shader_group_end(&self, group: &ShaderGroupRef) {
        unsafe {
            ffi::ShadingSystem_shader_group_end(self.ss, group.group);
        }
    }

    /// Set a parameter of the next shader that will be added to the group,
    /// optionally setting the 'lockgeom' metadata for that parameter
    /// (despite how it may have been set in the shader).  If lockgeom is
    /// false, it means that this parameter should NOT be considered locked
    /// against changes by the geometry, and therefore the shader should not
    /// optimize assuming that the instance value (the 'val' specified by
    /// this call) is a constant.
    // pub fn parameter(&self, group: &ShaderGroupRef, name: &str, t: TypeDesc)

    /// Append a new shader instance onto the specified group. The shader
    /// instance will get any pending parameters that were set by
    /// Parameter() calls since the last Shader() call for the group.
    pub fn shader(
        &self,
        group: &ShaderGroupRef,
        shaderusage: &str,
        shadername: &str,
        layername: &str,
    ) -> bool {
        let shaderusage = std::ffi::CString::new(shaderusage).unwrap();
        let shadername = std::ffi::CString::new(shadername).unwrap();
        let layername = std::ffi::CString::new(layername).unwrap();
        unsafe {
            ffi::ShadingSystem_shader(
                self.ss,
                group.group,
                shaderusage.as_ptr(),
                shadername.as_ptr(),
                layername.as_ptr(),
            )
        }
    }

    /// Create a per-thread data needed for shader execution.  It's very
    /// important for the app to never use a PerThreadInfo from more than
    /// one thread (and probably a good idea allocate only one PerThreadInfo
    /// for each renderer thread), and destroy it with destroy_thread_info
    /// when the thread terminates (and before the ShadingSystem is
    /// destroyed).
    pub fn create_thread_info(&self) -> PerThreadInfo {
        unsafe { ffi::ShadingSystem_create_thread_info(self.ss) }
    }

    /// Destroy a PerThreadInfo that was allocated by
    /// create_thread_info().
    pub fn destroy_thread_info(&self, tinfo: PerThreadInfo) {
        unsafe { ffi::ShadingSystem_destroy_thread_info(self.ss, tinfo) }
    }

    /// Get a ShadingContext that we can use.  The context is specific to a
    /// renderer thread, and should never be passed between or shared by
    /// more than one thread.  The 'threadinfo' parameter should be a
    /// thread-specific pointer created by create_thread_info.  The context
    /// can be used to shade many points; a typical usage is to allocate
    /// just one context per thread and use it for the whole run.
    pub fn get_context(&self, tinfo: PerThreadInfo) -> ShadingContext {
        unsafe { ffi::ShadingSystem_get_context(self.ss, tinfo) }
    }

    /// Return a ShadingContext to the pool.
    pub fn release_context(&self, context: ShadingContext) {
        unsafe {
            ffi::ShadingSystem_release_context(self.ss, context);
        }
    }

    /// Execute the shader group in this context. If ctx is NULL, then
    /// execute will request one (based on the running thread) on its own
    /// and then return it when it's done.  This is just a wrapper around
    /// execute_init, execute_layer of the last (presumably group entry)
    /// layer, and execute_cleanup. If run==false, just do the binding and
    /// setup, don't actually run the shader.
    pub fn execute(
        &self,
        context: ShadingContext,
        group: &ShaderGroupRef,
        sg: &ShaderGlobals,
        run: bool,
    ) -> bool {
        unsafe {
            ffi::ShadingSystem_execute(
                self.ss,
                context,
                group.group,
                sg as *const ShaderGlobals,
                run,
            )
        }
    }

    /// Search for an output symbol by name (and optionally, layer) within
    /// the optimized shader group. If the symbol is found, return an opaque
    /// identifying pointer to it, otherwise return NULL. This is somewhat
    /// expensive because of the name-based search, but once done, you can
    /// reuse the pointer to the symbol for the lifetime of the group.
    ///
    /// If you give just a symbol name, it will search for the symbol in all
    /// layers, last-to-first. If a specific layer is named, it will search
    /// only that layer. You can specify a layer either by naming it
    /// separately, or by concatenating "layername.symbolname", but note
    /// that the latter will involve string manipulation inside find_symbol
    /// and is much more expensive than specifying them separately.
    pub fn find_symbol(
        &self,
        group: &ShaderGroupRef,
        symbolname: Ustring,
    ) -> Result<ShaderSymbol, Error> {
        let symbol =
            unsafe { ffi::ShadingSystem_find_symbol(self.ss, group.group, symbolname.ptr) };

        if symbol.is_null() {
            Err(Error::SymbolNotFound(symbolname.to_string()))
        } else {
            Ok(ShaderSymbol { symbol })
        }
    }

    /// Given an opaque ShaderSymbol*, return the TypeDesc describing it.
    /// Note that a closure will end up with a TypeDesc::UNKNOWN value.
    pub fn symbol_typedesc(&self, symbol: ShaderSymbol) -> TypeDesc {
        unsafe { ffi::ShadingSystem_symbol_typedesc(self.ss, symbol.symbol) }
    }

    /// Given a context (that has executed a shader) and an opaque
    /// ShserSymbol*, return the actual memory address where the value of
    /// the symbol resides within the heap memory of the context. This
    /// is only valid for the shader execution that had happened immediately
    /// prior for this context, but it is a very inexpensive operation.
    pub fn symbol_address(
        &self,
        ctx: ShadingContext,
        symbol: ShaderSymbol,
    ) -> *const std::ffi::c_void {
        unsafe { ffi::ShadingSystem_symbol_address(self.ss, ctx, symbol.symbol) }
    }

    pub fn shade_image(
        &self,
        group: &ShaderGroupRef,
        defaultsg: Option<&ShaderGlobals>,
        imagebuf: &ImageBuf,
        outputs: &[Ustring],
        shadelocations: i32,
        roi: ROI,
    ) -> Result<(), Error> {
        unsafe {
            let defaultsg = if let Some(sg) = defaultsg {
                sg as *const ShaderGlobals
            } else {
                std::ptr::null()
            };
            if ffi::shade_image(
                self.ss,
                group.group,
                defaultsg,
                imagebuf.buf,
                outputs.as_ptr(),
                outputs.len() as i32,
                shadelocations,
                roi,
            ) {
                Ok(())
            } else {
                Err(Error::ShadeImageFailed)
            }
        }
    }
}

impl Drop for ShadingSystem {
    fn drop(&mut self) {
        unsafe {
            ffi::ShadingSystem_destroy(self.ss);
        }
    }
}

pub struct ShaderGroup {
    group: ffi::ShaderGroupRef,
}

impl Drop for ShaderGroup {
    fn drop(&mut self) {
        unsafe {
            ffi::ShaderGroup_destroy(self.group);
        }
    }
}

pub type ShaderGroupRef = Arc<ShaderGroup>;

pub struct ShaderSymbol {
    symbol: ffi::ShaderSymbolPtr,
}

/// The ShaderGlobals structure represents the state describing a particular
/// point to be shaded. It serves two primary purposes: (1) it holds the
/// values of the "global" variables accessible from a shader (such as P, N,
/// Ci, etc.); (2) it serves as a means of passing (via opaque pointers)
/// additional state between the renderer when it invokes the shader, and
/// the RendererServices that fields requests from OSL back to the renderer.
///
/// Except where noted, it is expected that all values are filled in by the
/// renderer before passing it to ShadingSystem::execute() to actually run
/// the shader. Not all fields will be valid in all contexts. In particular,
/// a few are only needed for lights and volumes.
///
/// All points, vectors and normals are given in "common" space.
///
pub struct ShaderGlobals {
    /// Surface position (and its x & y differentials).
    pub P: V3f32,
    pub dPdx: V3f32,
    pub dPdy: V3f32,
    /// P's z differential, used for volume shading only.
    pub dPdz: V3f32,

    /// Incident ray, and its x and y derivatives.
    pub I: V3f32,
    pub dIdx: V3f32,
    pub dIdy: V3f32,

    /// Shading normal, already front-facing.
    pub N: V3f32,

    /// True geometric normal.
    pub Ng: V3f32,

    /// 2D surface parameter u, and its differentials.
    pub u: f32,
    pub dudx: f32,
    pub dudy: f32,
    /// 2D surface parameter v, and its differentials.
    pub v: f32,
    pub dvdx: f32,
    pub dvdy: f32,

    /// Surface tangents: derivative of P with respect to surface u and v.
    pub dPdu: V3f32,
    pub dPdv: V3f32,

    /// Time for this shading sample.
    pub time: f32,
    /// Time interval for the frame (or shading sample).
    pub dtime: f32,
    ///  Velocity vector: derivative of position P with respect to time.
    pub dPdtime: V3f32,

    /// For lights or light attenuation shaders: the point being illuminated
    /// (Ps), and its differentials.
    pub Ps: V3f32,
    pub dPsdx: V3f32,
    pub dPsdy: V3f32,

    /// There are three opaque pointers that may be set by the renderer here
    /// in the ShaderGlobals before shading execution begins, and then
    /// retrieved again from the within the implementation of various
    /// RendererServices methods. Exactly what they mean and how they are
    /// used is renderer-dependent, but roughly speaking it's probably a
    /// pointer to some internal renderer state (needed for, say, figuring
    /// out how to retrieve userdata), state about the ray tree (needed to
    /// resume for a trace() call), and information about the object being
    /// shaded.
    pub renderstate: *const std::ffi::c_void,
    pub tracedata: *const std::ffi::c_void,
    pub objdata: *const std::ffi::c_void,

    /// Back-pointer to the ShadingContext (set and used by OSL itself --
    /// renderers shouldn't mess with this at all).
    pub context: ShadingContext,

    /// Pointer to the RendererServices object. This is how OSL finds its
    /// way back to the renderer for callbacks.
    pub renderer: ffi::RendererServicesWrapper,

    /// Opaque pointers set by the renderer before shader execution, to
    /// allow later retrieval of the object->common and shader->common
    /// transformation matrices, by the RendererServices
    /// get_matrix/get_inverse_matrix methods. This doesn't need to point
    /// to the 4x4 matrix itself; rather, it's just a pointer to whatever
    /// structure the RenderServices::get_matrix() needs to (if and when
    /// requested) generate the 4x4 matrix for the right time value.
    pub object2common: *const std::ffi::c_void,
    pub shader2common: *const std::ffi::c_void,

    /// The output closure will be placed here. The rendererer should
    /// initialize this to NULL before shading execution, and this is where
    /// it can retrieve the output closure from after shader execution has
    /// completed.
    pub Ci: ffi::ClosureColorPtr,

    /// Surface area of the emissive object (used by light shaders for
    /// energy normalization).
    pub surfacearea: f32,

    /// Bit field of ray type flags.
    pub raytype: i32,

    /// If nonzero, will flip the result of calculatenormal().
    pub flipHandedness: i32,

    /// If nonzero, we are shading the back side of a surface.
    pub backfacing: i32,
}

impl ShaderGlobals {
    pub fn new(context: ShadingContext, renderer: RendererServicesWrapper) -> ShaderGlobals {
        ShaderGlobals {
            P: v3f32(0.0, 0.0, 0.0),
            dPdx: v3f32(0.0, 0.0, 0.0),
            dPdy: v3f32(0.0, 0.0, 0.0),
            dPdz: v3f32(0.0, 0.0, 0.0),

            I: v3f32(0.0, 0.0, 0.0),
            dIdx: v3f32(0.0, 0.0, 0.0),
            dIdy: v3f32(0.0, 0.0, 0.0),

            N: v3f32(0.0, 0.0, 0.0),
            Ng: v3f32(0.0, 0.0, 0.0),

            u: 0.0,
            dudx: 0.0,
            dudy: 0.0,

            v: 0.0,
            dvdx: 0.0,
            dvdy: 0.0,

            dPdu: v3f32(0.0, 0.0, 0.0),
            dPdv: v3f32(0.0, 0.0, 0.0),

            time: 0.0,
            dtime: 0.0,
            dPdtime: v3f32(0.0, 0.0, 0.0),

            Ps: v3f32(0.0, 0.0, 0.0),
            dPsdx: v3f32(0.0, 0.0, 0.0),
            dPsdy: v3f32(0.0, 0.0, 0.0),

            renderstate: std::ptr::null(),
            tracedata: std::ptr::null(),
            objdata: std::ptr::null(),

            context,
            renderer,

            object2common: std::ptr::null(),
            shader2common: std::ptr::null(),

            Ci: std::ptr::null_mut(),

            surfacearea: 0.0,
            raytype: 0,
            flipHandedness: 0,
            backfacing: 0,
        }
    }
}

#[derive(Debug)]
pub enum Error {
    SymbolNotFound(String),
    ShadeImageFailed,
}

pub extern "C" fn handle_errors(level: i32, msg: *const std::os::raw::c_char) {
    let msg = unsafe {
        std::ffi::CStr::from_ptr(msg)
            .to_string_lossy()
            .into_owned()
            .to_string()
    };
    if level == ErrCode::Debug as i32 {
        println!("DEBUG: {}", msg);
    } else if level == ErrCode::Info as i32 {
        println!("INFO: {}", msg);
    } else if level == ErrCode::Message as i32 {
        println!("MESSAGE: {}", msg);
    } else if level == ErrCode::Warning as i32 {
        println!("WARNING: {}", msg);
    } else if level == ErrCode::Error as i32 {
        println!("ERROR: {}", msg);
    } else if level == ErrCode::Severe as i32 {
        println!("SEVERE: {}", msg);
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[repr(i32)]
    enum ClosureID {
        Emission = 0,
        Diffuse,
        Microfacet,
    }

    #[repr(C)]
    struct EmptyParams {}

    #[repr(C)]
    struct DiffuseParams {
        N: V3f32,
    }

    #[repr(C)]
    struct MicrofacetParams {
        dist: Ustring,
        N: V3f32,
        U: V3f32,
        xalpha: f32,
        yalpha: f32,
        eta: f32,
        refract: i32,
    }

    #[test]
    fn it_works() {
        unsafe {
            let width = 512;
            let height = 512;
            let mut renderer = TestRenderer::new(width, height);
            let c: Arc<RefCell<TestRenderer>> = Arc::clone(&renderer);
            let rs: Arc<RefCell<dyn RendererServices>> = c;
            let mut ss = ShadingSystem::new(rs);

            let mut closure_params = Vec::new();
            let mut offset = 0;

            closure_params.push(ClosureParam {
                typedesc: oiio::typedesc::STRING,
                offset,
                key: None,
                field_size: std::mem::size_of::<Ustring>(),
            });
            offset += std::mem::size_of::<String>();
            println!("sizeof String: {}", std::mem::size_of::<String>());
            println!("alignof String: {}", std::mem::align_of::<String>());
            println!("offset: {}", offset);

            closure_params.push(ClosureParam {
                typedesc: oiio::typedesc::NORMAL,
                offset,
                key: None,
                field_size: std::mem::size_of::<V3f32>(),
            });
            offset += std::mem::size_of::<V3f32>();
            println!("sizeof V3f32: {}", std::mem::size_of::<V3f32>());
            println!("alignof V3f32: {}", std::mem::align_of::<V3f32>());
            println!("offset: {}", offset);

            closure_params.push(ClosureParam {
                typedesc: oiio::typedesc::VECTOR,
                offset,
                key: None,
                field_size: std::mem::size_of::<V3f32>(),
            });
            offset += std::mem::size_of::<V3f32>();
            println!("offset: {}", offset);

            // xalpha
            closure_params.push(ClosureParam {
                typedesc: oiio::typedesc::FLOAT,
                offset,
                key: None,
                field_size: std::mem::size_of::<f32>(),
            });
            offset += std::mem::size_of::<f32>();

            // yalpha
            closure_params.push(ClosureParam {
                typedesc: oiio::typedesc::FLOAT,
                offset,
                key: None,
                field_size: std::mem::size_of::<f32>(),
            });
            offset += std::mem::size_of::<f32>();

            // eta
            closure_params.push(ClosureParam {
                typedesc: oiio::typedesc::FLOAT,
                offset,
                key: None,
                field_size: std::mem::size_of::<f32>(),
            });
            offset += std::mem::size_of::<f32>();

            // refract
            closure_params.push(ClosureParam {
                typedesc: oiio::typedesc::INT32,
                offset,
                key: None,
                field_size: std::mem::size_of::<i32>(),
            });
            offset += std::mem::size_of::<f32>();

            // finish
            closure_params.push(ClosureParam {
                typedesc: oiio::typedesc::UNKNOWN,
                offset: std::mem::size_of::<MicrofacetParams>(),
                key: None,
                field_size: std::mem::align_of::<MicrofacetParams>(),
            });

            renderer.borrow_mut().init_shading_system(&ss);

            // Register the layout of all closures known to this renderer
            // Any closure used by the shader which is not registered, or
            // registered with a different number of arguments will lead
            // to a runtime error.
            ss.register_closure("microfacet", ClosureID::Microfacet as i32, &closure_params);

            // Remember that each shader parameter may optionally have a
            // metadata hint [[int lockgeom=...]], where 0 indicates that the
            // parameter may be overridden by the geometry itself, for example
            // with data interpolated from the mesh vertices, and a value of 1
            // means that it is "locked" with respect to the geometry (i.e. it
            // will not be overridden with interpolated or
            // per-geometric-primitive data).
            //
            // In order to most fully optimize the shader, we typically want any
            // shader parameter not explicitly specified to default to being
            // locked (i.e. no per-geometry override):
            assert!(ss.attribute("lockgeom", 1i32));

            // Now we declare our shader.
            //
            // Each material in the scene is comprised of a "shader group."
            // Each group is comprised of one or more "layers" (a.k.a. shader
            // instances) with possible connections from outputs of
            // upstream/early layers into the inputs of downstream/later layers.
            // A shader instance is the combination of a reference to a shader
            // master and its parameter values that may override the defaults in
            // the shader source and may be particular to this instance (versus
            // all the other instances of the same shader).
            //
            // A shader group declaration typically looks like this:
            //
            //   ShaderGroupRef group = ss->ShaderGroupBegin ();
            //   ss->Parameter (*group, "paramname", TypeDesc paramtype, void *value);
            //      ... and so on for all the other parameters of...
            //   ss->Shader (*group, "shadertype", "shadername", "layername");
            //      The Shader() call creates a new instance, which gets
            //      all the pending Parameter() values made right before it.
            //   ... and other shader instances in this group, interspersed with...
            //   ss->ConnectShaders (*group, "layer1", "param1", "layer2", "param2");
            //   ... and other connections ...
            //   ss->ShaderGroupEnd (*group);
            //
            // It looks so simple, and it really is, except that the way this
            // testshade program works is that all the Parameter() and Shader()
            // calls are done inside getargs(), as it walks through the command
            // line arguments, whereas the connections accumulate and have
            // to be processed at the end.  Bear with us.

            // Start the shader group and grab a reference to it.
            let group_name = "";
            let shadergroup = ss.shader_group_begin(group_name);

            // Set shader parameters and create shader
            ss.shader(&shadergroup, "surface", "noisetest", "");

            // set the group name as an attribute for some reason?
            // ...

            // End the group definition
            ss.shader_group_end(&shadergroup);

            // Add the shaders to the renderer
            renderer.borrow_mut().shaders.push(Arc::clone(&shadergroup));

            // Set up transformations
            // ...

            // set up output images
            let output_vars = vec!["Cout".to_string()];
            if !output_vars.is_empty() {
                // tell shading system which outputs we want
                // potentially outputs from a particular group, we'll ignore
                // that for now
                // FIXME: testshade converts to ustrings here before passing the
                // raw string to ShadingSystem. Should we do the same?
                ss.attribute("renderer_outputs", output_vars.as_slice());
            }

            let entry_layers = Vec::<String>::new();
            if !entry_layers.is_empty() {
                ss.attribute("entry_layers", entry_layers.as_slice());
            }

            let per_thread_info = ss.create_thread_info();
            let ctx = ss.get_context(per_thread_info);

            let sg = ShaderGlobals::new(ctx, renderer.borrow().rsw);
            // set all the stuff on the shader globals here
            // ...

            // Because we can only call find_symbol or get_symbol on something that
            // has been set up to shade (or executed), we call execute() but tell it
            // not to actually run the shader.
            ss.execute(ctx, &shadergroup, &sg, false);

            // Find the symbol we want to output
            let sym_cout = ss
                .find_symbol(&shadergroup, Ustring::new("Cout"))
                .expect("Could not find Cout symbol");

            let sym_type = ss.symbol_typedesc(sym_cout);
            println!("Symbol Cout is {:?}", sym_type);

            // TODO: We should be taking the outputs in from command line and potentially
            // have many...
            renderer.borrow_mut().add_output(
                "Cout",
                "Cout.exr",
                TypeDesc::from_basetype(sym_type.basetype),
                sym_type.base_values() as i32,
            );

            // release these for now since we've done setting up
            // FIXME: these should be getting consumed, and have their lifetime tied
            // to the ShadingSystem if not the ShaderGroup
            ss.release_context(ctx);
            ss.destroy_thread_info(per_thread_info);

            renderer.borrow_mut().prepare_render();

            renderer.borrow_mut().warmup();

            let roi = ROI::new(0, width, 0, height);
            let outputs = vec![Ustring::new("Cout")];

            #[cfg(feature = "optix")]
            renderer.render(width, height);
            #[cfg(not(feature = "optix"))]
            ss.shade_image(
                &shadergroup,
                Some(&sg),
                &renderer.borrow().output_bufs[0],
                outputs.as_slice(),
                0,
                roi,
            )
            .expect("Shade image failed");

            // copy result to host
            renderer.borrow_mut().finalize_pixel_buffer();

            // write image to disk
            let output_name = renderer.borrow().output_bufs[0].name();
            renderer.borrow().output_bufs[0].write(&output_name, typedesc::FLOAT);
        }
    }
}
