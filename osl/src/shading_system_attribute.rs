use std::os::raw::c_void;

use oiio::typedesc;
use oiio::typedesc::TypeDesc;

use crate::ffi;
use crate::shading_system::ShaderGroupRef;

pub trait ShadingSystemAttribute {
    const TYPEDESC: TypeDesc;

    fn set_attribute(&self, name: &str, ss: ffi::ShadingSystem) -> bool;
    fn set_group_attribute(
        &self,
        name: &str,
        ss: ffi::ShadingSystem,
        group: &ShaderGroupRef,
    ) -> bool;
}

impl ShadingSystemAttribute for i32 {
    const TYPEDESC: TypeDesc = typedesc::INT32;

    fn set_attribute(&self, name: &str, ss: ffi::ShadingSystem) -> bool {
        let name = std::ffi::CString::new(name).unwrap();
        unsafe {
            ffi::ShadingSystem_attribute(
                ss,
                name.as_ptr(),
                Self::TYPEDESC,
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
                Self::TYPEDESC,
                self as *const i32 as *const c_void,
            )
        }
    }
}

impl ShadingSystemAttribute for &[i32] {
    const TYPEDESC: TypeDesc = typedesc::INT32;

    fn set_attribute(&self, name: &str, ss: ffi::ShadingSystem) -> bool {
        let name = std::ffi::CString::new(name).unwrap();
        unsafe {
            ffi::ShadingSystem_attribute(
                ss,
                name.as_ptr(),
                TypeDesc::new(
                    Self::TYPEDESC.basetype,
                    Self::TYPEDESC.aggregate,
                    Self::TYPEDESC.vecsemantics,
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
                    Self::TYPEDESC.basetype,
                    Self::TYPEDESC.aggregate,
                    Self::TYPEDESC.vecsemantics,
                    self.len() as i32,
                ),
                self.as_ptr() as *const i32 as *const c_void,
            )
        }
    }
}

impl ShadingSystemAttribute for f32 {
    const TYPEDESC: TypeDesc = typedesc::FLOAT;

    fn set_attribute(&self, name: &str, ss: ffi::ShadingSystem) -> bool {
        let name = std::ffi::CString::new(name).unwrap();
        unsafe {
            ffi::ShadingSystem_attribute(
                ss,
                name.as_ptr(),
                Self::TYPEDESC,
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
                Self::TYPEDESC,
                self as *const f32 as *const c_void,
            )
        }
    }
}

impl ShadingSystemAttribute for &[f32] {
    const TYPEDESC: TypeDesc = typedesc::FLOAT;

    fn set_attribute(&self, name: &str, ss: ffi::ShadingSystem) -> bool {
        let name = std::ffi::CString::new(name).unwrap();
        unsafe {
            ffi::ShadingSystem_attribute(
                ss,
                name.as_ptr(),
                TypeDesc::new(
                    Self::TYPEDESC.basetype,
                    Self::TYPEDESC.aggregate,
                    Self::TYPEDESC.vecsemantics,
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
                    Self::TYPEDESC.basetype,
                    Self::TYPEDESC.aggregate,
                    Self::TYPEDESC.vecsemantics,
                    self.len() as i32,
                ),
                self.as_ptr() as *const f32 as *const c_void,
            )
        }
    }
}

impl ShadingSystemAttribute for &str {
    const TYPEDESC: TypeDesc = typedesc::STRING;

    fn set_attribute(&self, name: &str, ss: ffi::ShadingSystem) -> bool {
        let name = std::ffi::CString::new(name).unwrap();
        let value = std::ffi::CString::new(*self).unwrap();
        let value = [value.as_ptr()]; // OSL expects a **char
        unsafe {
            ffi::ShadingSystem_attribute(
                ss,
                name.as_ptr(),
                Self::TYPEDESC,
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
        let value = [value.as_ptr()]; // OSL expects a **char
        unsafe {
            ffi::ShadingSystem_group_attribute(
                ss,
                group.group,
                name.as_ptr(),
                Self::TYPEDESC,
                value.as_ptr() as *const c_void,
            )
        }
    }
}

impl ShadingSystemAttribute for &[String] {
    const TYPEDESC: TypeDesc = typedesc::STRING;

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
                    Self::TYPEDESC.basetype,
                    Self::TYPEDESC.aggregate,
                    Self::TYPEDESC.vecsemantics,
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
                    Self::TYPEDESC.basetype,
                    Self::TYPEDESC.aggregate,
                    Self::TYPEDESC.vecsemantics,
                    self.len() as i32,
                ),
                value_ptrs.as_ptr() as *const c_void,
            )
        }
    }
}

impl ShadingSystemAttribute for &[&str] {
    const TYPEDESC: TypeDesc = typedesc::STRING;

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
                    Self::TYPEDESC.basetype,
                    Self::TYPEDESC.aggregate,
                    Self::TYPEDESC.vecsemantics,
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
                    Self::TYPEDESC.basetype,
                    Self::TYPEDESC.aggregate,
                    Self::TYPEDESC.vecsemantics,
                    self.len() as i32,
                ),
                value_ptrs.as_ptr() as *const c_void,
            )
        }
    }
}
