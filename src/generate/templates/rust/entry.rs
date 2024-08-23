/// Entry point: {entry_name}
pub fn {entry_name}<'a>(ctx: &'a Context, {entry_params}) -> Result<{entry_return_type}, Error> {{
    {out_decl}
    let rc = unsafe {{
        futhark_entry_{entry_name}(ctx.context, {call_args})
    }};
    if rc != 0 {{ return Err(Error::Code(rc)); }}

    #[allow(unused_unsafe)]
    unsafe {{
        Ok({entry_return})
    }}
}}

extern "C" {{
    fn {entry_fn}(
        _: *mut futhark_context,
        {futhark_entry_params}
    ) -> core::ffi::c_int;
}}
