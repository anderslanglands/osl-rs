mod ffi;
use ffi::{ErrCode, PerThreadInfo, RendererServicesWrapper, ShadingContext, VerbosityLevel};
pub mod math;
pub use math::*;

pub mod renderer_services;
pub use renderer_services::*;

pub mod shader_globals;
pub use shader_globals::*;

pub mod shading_system;
pub use shading_system::*;

pub mod shading_system_attribute;
pub use shading_system_attribute::*;

pub mod closure;
pub use closure::*;

mod test_renderer;
use test_renderer::TestRenderer;

use std::os::raw::{c_char, c_void};

use std::cell::RefCell;
use std::sync::Arc;

use oiio::imagebuf::ImageBuf;
use oiio::imageio::{ImageSpec, ROI};
pub use oiio::typedesc;
pub use oiio::typedesc::TypeDesc;
pub use oiio::Ustring;

#[macro_use]
extern crate derive_more;

#[derive(Debug, Display)]
pub enum Error {
    #[display(fmt = "PerThreadInfo creation failed")]
    ThreadInfoFailed,
    #[display(fmt = "Failed to create shader '{}' '{}' '{}'", _0, _1, _2)]
    ShaderFailed(String, String, String),
    #[display(fmt = "Failed to set group attribute '{}' on shading system", _0)]
    SetGroupAttributeFailed(String),
    #[display(fmt = "Failed to set attribute '{}' on shading system", _0)]
    SetAttributeFailed(String),
    #[display(fmt = "Symbol '{}' not found", _0)]
    SymbolNotFound(String),
    #[display(fmt = "shade_image failed")]
    ShadeImageFailed,
    #[display(fmt = "Failed to get shading context")]
    GetShadingContextFailed,
    #[display(fmt = "Failed to execute shading group")]
    ExecuteFailed,
}

#[cfg(test)]
mod tests {
    use crate::*;

    use osl_derive::Closure;

    #[repr(i32)]
    enum ClosureID {
        Emission = 0,
        Diffuse,
        Microfacet,
    }

    #[repr(C)]
    #[derive(Closure)]
    #[name = "emission"]
    #[id = 0]
    struct EmissionParams {}

    #[repr(C)]
    #[derive(Closure)]
    #[name = "diffuse"]
    #[id = 1]
    struct DiffuseParams {
        #[vecsemantics = "NORMAL"]
        N: V3f32,
    }

    #[repr(C)]
    #[derive(Closure)]
    #[name = "microfacet"]
    #[id = 2]
    struct MicrofacetParams {
        dist: Ustring,
        #[vecsemantics = "NORMAL"]
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

            renderer.borrow_mut().init_shading_system(&ss);

            // Register the layout of all closures known to this renderer
            // Any closure used by the shader which is not registered, or
            // registered with a different number of arguments will lead
            // to a runtime error.
            MicrofacetParams::register_with(&mut ss);
            DiffuseParams::register_with(&mut ss);
            EmissionParams::register_with(&mut ss);

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
            ss.attribute("lockgeom", 1i32)
                .expect("Could not set lockgeom");

            ss.attribute("searchpath:shader", "osl")
                .expect("Could not set searchpath");

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
            ss.shader(&shadergroup, "surface", "noisetest", "")
                .expect("Shader creation failed");

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
                ss.attribute("renderer_outputs", output_vars.as_slice())
                    .expect("Failed to set renderer_outputs attribute");
            }

            let entry_layers = Vec::<String>::new();
            if !entry_layers.is_empty() {
                ss.attribute("entry_layers", entry_layers.as_slice())
                    .expect("failed to set entry_layers attribute");
            }

            let per_thread_info = ss
                .create_thread_info()
                .expect("Could not create per-thread info");
            let ctx = ss
                .get_context(per_thread_info)
                .expect("Could not create context");

            let sg = ShaderGlobals::new(ctx, renderer.borrow().rsw);
            // set all the stuff on the shader globals here
            // ...

            // Because we can only call find_symbol or get_symbol on something that
            // has been set up to shade (or executed), we call execute() but tell it
            // not to actually run the shader.
            ss.execute(ctx, &shadergroup, &sg, false)
                .expect("Execute failed");

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
            renderer.borrow().output_bufs[0]
                .write(&output_name, typedesc::FLOAT)
                .expect("Could not write image");
        }
    }
}
