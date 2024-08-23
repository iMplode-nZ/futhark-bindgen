impl<'a> {rust_type}<'a> {{
    /// Create new {rust_type}
    pub fn new(ctx: &'a Context, {new_params}) -> Self {{
        unsafe {{
            let mut out = std::ptr::null_mut();
            let rc = {new_fn}(ctx.context, &mut out, {new_call_args});
            if rc != 0 {{ panic!("{rust_type}::new creation failed with error code {{rc}}"); }}
            ctx.auto_sync();
            Self {{ ptr: out, ctx }}
        }}
    }}

    /// Create new {rust_type}, returning an error if the operation fails
    pub fn new_checked(ctx: &'a Context, {new_params}) -> Result<Self, Error> {{
        unsafe {{
            let mut out = std::ptr::null_mut();
            let rc = {new_fn}(ctx.context, &mut out, {new_call_args});
            if rc != 0 {{ return Err(Error::Code(rc)); }}
            ctx.auto_sync();
            Ok(Self {{ ptr: out, ctx }})
        }}
    }}
}}

extern "C" {{
    fn {new_fn}(
        ctx: *mut futhark_context,
        _: *mut *mut {raw_type},
        {new_extern_params}
    ) -> core::ffi::c_int;
}}
