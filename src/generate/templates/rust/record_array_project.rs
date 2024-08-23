impl<'a> {rust_type}<'a> {{
    pub fn {project_name}(&self) -> {rust_field_type} {{
        self.{project_name}_checked().unwrap()
    }}
    pub fn {project_name}_checked(&self) -> Result<{rust_field_type}, Error> {{
        unsafe {{
            let mut out = std::ptr::null_mut();
            let rc = {project_fn}(self.ctx.context, &mut out, self.ptr);
            if rc != 0 {{ return Err(Error::Code(rc)); }}
            self.ctx.auto_sync();
            Ok({rust_field_type}::from_ptr(self.ctx, out))
        }}
    }}
}}

extern "C" {{
    fn {project_fn}(
        ctx: *mut futhark_context,
        out: *mut *mut {raw_field_type},
        obj: *const {raw_type},
    ) -> core::ffi::c_int;
}}
