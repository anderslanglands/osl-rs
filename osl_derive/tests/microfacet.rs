use osl::*;
use osl_derive::Closure;

#[repr(C)]
#[derive(Closure)]
#[name="microfacet"]
#[id=0]
struct MicrofacetParams {
    dist: Ustring,
    #[vecsemantics="NORMAL"]
    N: V3f32,
    U: V3f32,
    xalpha: f32,
    yalpha: f32,
    eta: f32,
    refract: i32,
}

