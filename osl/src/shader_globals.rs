use crate::ffi;
use crate::math::*;
use ffi::{ErrCode, PerThreadInfo, RendererServicesWrapper, ShadingContext, VerbosityLevel};

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
#[repr(C)]
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
