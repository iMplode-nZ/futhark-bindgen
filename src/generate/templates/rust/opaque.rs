#[repr(C)]
#[allow(non_camel_case_types)]
struct {raw_type} {{
    _private: [u8; 0]
}}

/// Futhark type
pub struct {rust_type}<'a> {{
    ptr: *mut {raw_type},
    ctx: &'a Context,
}}

impl<'a> {rust_type}<'a> {{
    #[allow(unused)]
    fn from_ptr(ctx: &'a Context, ptr: *mut {raw_type}) -> Self {{
        Self {{ ctx, ptr }}
    }}
}}

impl<'a> Drop for {rust_type}<'a> {{
    fn drop(&mut self) {{
        unsafe {{
            {free_fn}(self.ctx.context, self.ptr);
        }}
    }}
}}

extern "C" {{
    fn {free_fn}(
        ctx: *mut futhark_context,
        obj: *mut {raw_type}
    ) -> core::ffi::c_int;
    #[allow(unused)]
    fn {store_fn}(
        ctx: *mut futhark_context,
        obj: *const {raw_type},
        p: *mut *mut core::ffi::c_void,
        n: *mut usize // c_size_t
    ) -> core::ffi::c_int;
    #[allow(unused)]
    fn {restore_fn}(
        ctx: *mut futhark_context,
        p: *const core::ffi::c_void,
    ) -> {raw_type};
}}
