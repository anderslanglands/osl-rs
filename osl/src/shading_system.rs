use crate::closure::ClosureParam;
use crate::ffi;
use crate::ffi::{ErrCode, PerThreadInfo, ShadingContext};
use crate::renderer_services::RendererServices;
use crate::shader_globals::ShaderGlobals;
use crate::shading_system_attribute::ShadingSystemAttribute;
use crate::Error;

use std::cell::RefCell;
use std::sync::Arc;

use oiio::imagebuf::ImageBuf;
use oiio::imageio::ROI;
use oiio::typedesc::TypeDesc;
use oiio::Ustring;

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
    pub fn attribute<T: ShadingSystemAttribute>(
        &mut self,
        name: &str,
        val: T,
    ) -> Result<(), Error> {
        if val.set_attribute(name, self.ss) {
            Ok(())
        } else {
            Err(Error::SetAttributeFailed(name.into()))
        }
    }

    pub fn group_attribute<T: ShadingSystemAttribute>(
        &mut self,
        group: &ShaderGroupRef,
        name: &str,
        val: T,
    ) -> Result<(), Error> {
        if val.set_group_attribute(name, self.ss, group) {
            Ok(())
        } else {
            Err(Error::SetGroupAttributeFailed(name.into()))
        }
    }

    // FIXME: Handle potential null case
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
    ) -> Result<(), Error> {
        let cshaderusage = std::ffi::CString::new(shaderusage).unwrap();
        let cshadername = std::ffi::CString::new(shadername).unwrap();
        let clayername = std::ffi::CString::new(layername).unwrap();
        if unsafe {
            ffi::ShadingSystem_shader(
                self.ss,
                group.group,
                cshaderusage.as_ptr(),
                cshadername.as_ptr(),
                clayername.as_ptr(),
            )
        } {
            Ok(())
        } else {
            Err(Error::ShaderFailed(
                shaderusage.into(),
                shadername.into(),
                layername.into(),
            ))
        }
    }

    /// Create a per-thread data needed for shader execution.  It's very
    /// important for the app to never use a PerThreadInfo from more than
    /// one thread (and probably a good idea allocate only one PerThreadInfo
    /// for each renderer thread), and destroy it with destroy_thread_info
    /// when the thread terminates (and before the ShadingSystem is
    /// destroyed).
    pub fn create_thread_info(&self) -> Result<PerThreadInfo, Error> {
        let tinfo = unsafe { ffi::ShadingSystem_create_thread_info(self.ss) };
        if tinfo.is_null() {
            Err(Error::ThreadInfoFailed)
        } else {
            Ok(tinfo)
        }
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
    pub fn get_context(&self, tinfo: PerThreadInfo) -> Result<ShadingContext, Error> {
        let ctx = unsafe { ffi::ShadingSystem_get_context(self.ss, tinfo) };
        if ctx.is_null() {
            Err(Error::GetShadingContextFailed)
        } else {
            Ok(ctx)
        }
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
    ) -> Result<(), Error> {
        if unsafe {
            ffi::ShadingSystem_execute(
                self.ss,
                context,
                group.group,
                sg as *const ShaderGlobals,
                run,
            )
        } {
            Ok(())
        } else {
            Err(Error::ExecuteFailed)
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
    pub group: ffi::ShaderGroupRef,
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
