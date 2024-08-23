/// Futhark type
pub struct {rust_type}<'a> {{
    ptr: *mut {raw_type},
    shape: [usize; {rank}],
    ctx: &'a Context,
}}


impl<'a> FutharkArray for {rust_type}<'a> {{
    const RANK: usize = {rank};
    type Element = {rust_elemtype}<'a>;
}}

impl<'a> {rust_type}<'a> {{
    pub fn shape(&self) -> [usize; {rank}] {{
        self.shape
    }}

    /// Gets the value at the given index, and syncs the context.
    pub fn get(&self, index: [usize; {rank}]) -> {rust_elemtype} {{
        self.get_checked(index).unwrap()
    }}

    /// Gets the value at the given index, and syncs the context.
    pub fn get_checked(&self, index: [usize; {rank}]) -> Result<{rust_elemtype}, Error> {{
        if index.iter().zip(self.shape.iter()).any(|(i, s)| *i >= *s) {{
            return Err(Error::IndexOutOfBounds);
        }}

        let mut out = std::ptr::null_mut();
        let rc = unsafe {{
            {index_fn}(self.ctx.context, &mut out, self.ptr, {index_params})
        }};
        if rc != 0 {{
            return Err(Error::Code(rc));
        }}
        self.ctx.sync();
        Ok({rust_elemtype}::from_ptr(self.ctx, out))
    }}

    #[allow(unused)]
    fn from_ptr(ctx: &'a Context, ptr: *mut {raw_type}) -> Self {{
        let len_ptr = unsafe {{ {shape_fn}(ctx.context, ptr) }};
        let mut shape = [0usize; {rank}];
        unsafe {{
            for (i, s) in shape.iter_mut().enumerate() {{
                *s = (*len_ptr.add(i)) as usize;
            }}
        }}

        Self {{ ctx, shape, ptr }}
    }}
}}

impl<'a> Drop for {rust_type}<'a> {{
    fn drop(&mut self) {{
        unsafe {{
            {free_fn}(self.ctx.context, self.ptr);
        }}
    }}
}}

#[repr(C)]
#[allow(non_camel_case_types)]
struct {raw_type} {{
    _private: [u8; 0]
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
    fn {shape_fn}(
        ctx: *mut futhark_context,
        arr: *mut {raw_type}
    ) -> *const i64;
    fn {index_fn}(
        ctx: *mut futhark_context,
        out: *mut *mut {raw_elemtype},
        arr: *mut {raw_type},
        {index_args}
    ) -> core::ffi::c_int;
}}
