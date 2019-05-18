#include <OpenImageIO/imagebufalgo.h>

#include <OSL/oslexec.h>
#include <OpenImageIO/errorhandler.h>
#include <OpenImageIO/imagebuf.h>

typedef struct OSL::ShadingSystem* ShadingSystem;
typedef struct OSL::RendererServices* RendererServicesBase;
typedef struct OSL::TextureSystem* TextureSystem;
typedef struct OSL::ShaderGlobals* ShaderGlobals;
typedef struct OSL::PerThreadInfo* PerThreadInfoPtr;
typedef struct OSL::ShadingContext* ShadingContextPtr;
typedef struct OSL::ShaderGlobals* ShaderGlobalsPtr;
typedef const struct OSL::ShaderSymbol* ShaderSymbolPtr;
typedef struct OIIO::ImageBuf* ImageBufPtr;

typedef const char* ustring;

struct TypeDesc {
    unsigned char basetype;
    unsigned char aggregate;
    unsigned char vecsemantics;
    unsigned char reserved;
    int arraylen;
};

struct ClosureParam {
    TypeDesc type;
    int offset;
    const char* key;
    int field_size;
};

typedef void (*ErrorHandlerImpl)(int, const char*);

class ErrorHandlerWrapper : public OIIO::ErrorHandler {
    ErrorHandlerImpl _error_handler_impl;

public:
    ErrorHandlerWrapper(ErrorHandlerImpl impl) : _error_handler_impl(impl) {}

    virtual void operator()(int error_code, const std::string& msg) {
        _error_handler_impl(error_code, msg.c_str());
    }
};

typedef struct ErrorHandlerWrapper* ErrorHandler;

typedef const void* TransformationPtr;

typedef int (*RSFn_supports)(void* rs_obj, const char* feature);
typedef int (*RSFn_get_matrix)(void* rs_obj, ShaderGlobals sg,
                               OSL::Matrix44* result, TransformationPtr xform);

class RendererServicesWrapperApi : public OSL::RendererServices {
public:
    void* _rs_obj;
    RSFn_supports _supports = nullptr;
    RSFn_get_matrix _get_matrix = nullptr;

    // RendererServicesWrapper(void* rs_obj) : _rs_obj(rs_obj) {}

    virtual int supports(const char* feature) {
        if (_supports) {
            return _supports(_rs_obj, feature);
        } else {
            return OSL::RendererServices::supports(feature);
        }
    }

    virtual bool get_matrix(ShaderGlobals sg, OSL::Matrix44& result,
                            TransformationPtr xform) {
        if (_get_matrix) {
            return _get_matrix(_rs_obj, sg, &result, xform);
        } else {
            return OSL::RendererServices::get_matrix(sg, result, xform);
        }
    }
};

typedef RendererServicesWrapperApi* RendererServicesWrapper;

struct ShaderGroupRefApi {
    ShaderGroupRefApi(std::shared_ptr<OSL::ShaderGroup> group) : group(group) {}
    std::shared_ptr<OSL::ShaderGroup> group;
};

typedef ShaderGroupRefApi* ShaderGroupRef;

extern "C" {

// FIXME: texture system
ShadingSystem ShadingSystem_create(RendererServicesWrapper renderer) {
    return new OSL::ShadingSystem(renderer, NULL, NULL);
}

ShadingSystem
ShadingSystem_create_with_error_handler(RendererServicesWrapper renderer,
                                        ErrorHandler eh) {
    return new OSL::ShadingSystem(renderer, NULL, eh);
}

void ShadingSystem_destroy(ShadingSystem ss) { delete ss; }

void ShadingSystem_register_closure(ShadingSystem ss, const char* name, int id,
                                    const ClosureParam* params) {
    ss->register_closure(name, id, (const OSL::ClosureParam*)params, NULL,
                         NULL);
}

bool ShadingSystem_attribute(ShadingSystem ss, const char* name,
                             TypeDesc typedesc, const void* val) {
    return ss->attribute(name, *(OIIO::TypeDesc*)&typedesc, val);
}

bool ShadingSystem_group_attribute(ShadingSystem ss, ShaderGroupRef group,
                                   const char* name, TypeDesc typedesc,
                                   const void* val) {
    return ss->attribute(group->group.get(), name, *(OIIO::TypeDesc*)&typedesc,
                         val);
}

ShaderGroupRef ShadingSystem_shader_group_begin(ShadingSystem ss,
                                                const char* groupname) {
    return new ShaderGroupRefApi(ss->ShaderGroupBegin(groupname));
}

void ShadingSystem_shader_group_end(ShadingSystem ss, ShaderGroupRef group) {
    ss->ShaderGroupEnd(*group->group);
}

bool ShadingSystem_shader(ShadingSystem ss, ShaderGroupRef group,
                          const char* shaderusage, const char* shadername,
                          const char* layername) {
    return ss->Shader(*group->group, shaderusage, shadername, layername);
}

PerThreadInfoPtr ShadingSystem_create_thread_info(ShadingSystem ss) {
    return ss->create_thread_info();
}

void ShadingSystem_destroy_thread_info(ShadingSystem ss,
                                       PerThreadInfoPtr tinfo) {
    ss->destroy_thread_info(tinfo);
}

ShadingContextPtr ShadingSystem_get_context(ShadingSystem ss,
                                            PerThreadInfoPtr tinfo) {
    return ss->get_context(tinfo);
}

void ShadingSystem_release_context(ShadingSystem ss,
                                   ShadingContextPtr context) {
    ss->release_context(context);
}

bool ShadingSystem_execute(ShadingSystem ss, ShadingContextPtr ctx,
                           ShaderGroupRef group, ShaderGlobalsPtr sg,
                           bool run) {
    return ss->execute(*ctx, *group->group, *sg, run);
}

ShaderSymbolPtr ShadingSystem_find_symbol(ShadingSystem ss,
                                          ShaderGroupRef group,
                                          ustring symbolname) {
    return ss->find_symbol(*group->group, *(OIIO::ustring*)&symbolname);
}

TypeDesc ShadingSystem_symbol_typedesc(ShadingSystem ss,
                                       ShaderSymbolPtr symbol) {
    auto td = ss->symbol_typedesc(symbol);
    return *(TypeDesc*)&td;
}

const void* ShadingSystem_symbol_address(ShadingSystem ss,
                                         ShadingContextPtr ctx,
                                         ShaderSymbolPtr symbol) {
    return ss->symbol_address(*ctx, symbol);
}

void ShaderGroup_destroy(ShaderGroupRef group) { delete group; }

RendererServicesBase RendererServices_create() {
    return new OSL::RendererServices();
}

void RendererServices_destroy(RendererServicesBase rs) { delete rs; }

RendererServicesWrapper RendererServicesWrapper_create() {
    return new RendererServicesWrapperApi();
}

void RendererServicesWrapper_destroy(RendererServicesWrapper rsw) {
    delete rsw;
}

void RendererServicesWrapper_set_rust_object(RendererServicesWrapper rsw,
                                             void* rs_obj) {
    rsw->_rs_obj = rs_obj;
}

void RendererServicesWrapper_setfn_supports(RendererServicesWrapper rsw,
                                            RSFn_supports supports) {
    rsw->_supports = supports;
}

void RendererServicesWrapper_setfn_get_matrix(RendererServicesWrapper rsw,
                                              RSFn_get_matrix get_matrix) {
    rsw->_get_matrix = get_matrix;
}

ErrorHandler ErrorHandler_create(ErrorHandlerImpl impl) {
    return new ErrorHandlerWrapper(impl);
}

void ErrorHandler_destroy(ErrorHandler eh) { delete eh; }

void ErrorHandler_set_verbosity(ErrorHandler eh, int verbosity) {
    eh->verbosity(verbosity);
}

int ErrorHandler_get_verbosity(ErrorHandler eh) { return eh->verbosity(); }

bool shade_image(ShadingSystem ss, ShaderGroupRef group,
                 const ShaderGlobalsPtr defaultsg, ImageBufPtr imagebuf,
                 const ustring* outputs, int noutputs, int shadelocations,
                 OIIO::ROI roi) {
    return OSL::shade_image(
        *ss, *group->group, defaultsg, *imagebuf,
        OIIO::cspan<OIIO::ustring>((const OIIO::ustring*)outputs, noutputs),
        (OSL::ShadeImageLocations)shadelocations, roi);
}

} // extern "C"