#[repr(C)]
#[allow(non_camel_case_types)]
struct {raw_type} {{
    _private: [u8; 0]
}}

/// Array type with {rank} dimensions and {elemtype} elements
pub struct {rust_type}<'a> {{
    ptr: *mut {raw_type},
    shape: [usize; {rank}],
    ctx: &'a Context,
}}

impl<'a> FutharkArray for {rust_type}<'a> {{
    const RANK: usize = {rank};
    type Element = {elemtype};
}}

impl<'a> {rust_type}<'a> {{
    /// Create a new array of `dims` dimensions and initialize it with the values from `data`
    pub fn new(ctx: &'a Context, dims: [usize; {rank}], data: impl AsRef<[{elemtype}]>) -> Result<Self, Error> {{
        let size: usize = dims.iter().product();
        let data = data.as_ref();
        if data.len() != size {{
            return Err(Error::InvalidShape)
        }}
        let ptr = unsafe {{
            {new_fn}(ctx.context, data.as_ptr(), {dim_params})
        }};
        if ptr.is_null() {{ return Err(Error::NullPtr); }}
        ctx.auto_sync();
        Ok(Self {{
            ptr: ptr as *mut _,
            shape: dims,
            ctx,
        }})
    }}

    /// Get the array shape
    pub fn shape(&self) -> [usize; {rank}] {{
        self.shape
    }}

    /// Load values back into a slice
    pub fn values(&self, mut data: impl AsMut<[{elemtype}]>) -> Result<(), Error> {{
        let size: usize = self.shape.iter().product();
        let data = data.as_mut();
        if data.len() != size {{
            return Err(Error::InvalidShape);
        }}
        let rc = unsafe {{
            {values_fn}(self.ctx.context, self.ptr, data.as_mut_ptr())
        }};
        if rc != 0 {{
            return Err(Error::Code(rc));
        }}
        self.ctx.auto_sync();
        Ok(())
    }}

    /// Load values into a `Vec`
    pub fn as_vec(&self) -> Result<Vec<{elemtype}>, Error> {{
        let size: usize = self.shape.iter().product();
        let mut vec = vec![{elemtype}::default(); size];
        self.values(&mut vec)?;
        Ok(vec)
    }}

    /// Load the value at the given index into `out`.
    pub fn load_index(&self, index: [usize; {rank}], out: &mut {elemtype}) -> Result<(), Error> {{
        if index.iter().zip(self.shape.iter()).any(|(i, s)| *i >= *s) {{
            return Err(Error::IndexOutOfBounds);
        }}
        let rc = unsafe {{
            {index_fn}(self.ctx.context, out, self.ptr, {index_params})
        }};
        if rc != 0 {{
            return Err(Error::Code(rc));
        }}
        self.ctx.auto_sync();
        Ok(())
    }}

    /// Gets the value at the given index, and syncs the context. In order to avoid syncing, use `load_index` with `Context::auto_sync` set to false.
    pub fn get(&self, index: [usize; {rank}]) -> {elemtype} {{
        self.get_checked(index).unwrap()
    }}

    /// Gets the value at the given index, and syncs the context. In order to avoid syncing, use `load_index` with `Context::auto_sync` set to false.
    pub fn get_checked(&self, index: [usize; {rank}]) -> Result<{elemtype}, Error> {{
        let mut out = {elemtype}::default();
        self.load_index(index, &mut out)?;
        if !self.ctx.auto_sync {{
            self.ctx.sync();
        }}
        Ok(out)
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
    fn drop(&mut self){{
        unsafe {{
            {free_fn}(self.ctx.context, self.ptr as *mut _);
        }}
    }}
}}

#[allow(unused)]
extern "C" {{
    fn {shape_fn}(
        ctx: *mut futhark_context,
        arr: *mut {raw_type}
    ) -> *const i64;

    fn {new_fn}(
        ctx: *mut futhark_context,
        data: *const {elemtype},
        {new_dim_args}
    ) -> *mut {raw_type};

    fn {free_fn}(
        ctx: *mut futhark_context,
        arr: *mut {raw_type}
    ) -> core::ffi::c_int;

    fn {values_fn}(
        ctx: *mut futhark_context,
        arr: *mut {raw_type},
        data: *mut {elemtype}
    ) -> core::ffi::c_int;

    fn {index_fn}(
        ctx: *mut futhark_context,
        out: *mut {elemtype},
        arr: *mut {raw_type},
        {index_args}
    ) -> core::ffi::c_int;
}}
