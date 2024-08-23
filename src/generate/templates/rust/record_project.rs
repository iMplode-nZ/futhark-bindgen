impl<'a> {rust_type}<'a> {{
    pub fn {project_name}(&self) -> {rust_field_type} {{
        self.{project_name}_checked().unwrap()
    }}
    /// Get the {field_name} field
    pub fn {project_name}_checked(&self) -> Result<{rust_field_type}, Error> {{
        let mut out = std::mem::MaybeUninit::zeroed();
        let rc = unsafe {{
            {project_fn}(
                self.ctx.context,
                out.as_mut_ptr(),
                self.ptr
            )
        }};
        if rc != 0 {{ return Err(Error::Code(rc)); }}
        self.ctx.auto_sync();
        let out = unsafe {{ out.assume_init() }};
        {output}
    }}
}}

extern "C" {{
    fn {project_fn}(
        ctx: *mut futhark_context,
        _: *mut {raw_arg_type},
        _: *const {raw_type}
    ) -> core::ffi::c_int;
}}
