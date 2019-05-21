use crate::ffi;
use oiio::typedesc::TypeDesc;

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
