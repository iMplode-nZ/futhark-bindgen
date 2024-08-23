impl<'a> {rust_type}<'a> {{
    pub fn zip(ctx: &'a Context, {zip_params}) -> Self {{
        unsafe {{
            let mut out = std::ptr::null_mut();
            let rc = {zip_fn}(ctx.context, &mut out, {zip_call_args});
            if rc != 0 {{ panic!("{rust_type}::zip creation failed with error code {{rc}}"); }}
            ctx.auto_sync();
            Self::from_ptr(ctx, out)
        }}
    }}
    pub fn zip_checked(ctx: &'a Context, {zip_params}) -> Result<Self, Error> {{
        unsafe {{
            let mut out = std::ptr::null_mut();
            let rc = {zip_fn}(ctx.context, &mut out, {zip_call_args});
            if rc != 0 {{ return Err(Error::Code(rc)); }}
            ctx.auto_sync();
            Ok(Self::from_ptr(ctx, out))
        }}
    }}
}}

extern "C" {{
    fn {zip_fn}(
        ctx: *mut futhark_context,
        out: *mut *mut {raw_type},
        {zip_extern_params}
    ) -> core::ffi::c_int;
}}

