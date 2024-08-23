impl Context {{
    /// Entry point: {entry_name}
    pub fn {entry_name}<'a>(&'a self, {entry_params}) -> Result<{entry_return_type}, Error> {{
        #[allow(unused)]
        let ctx = self;
        {out_decl}
        let rc = unsafe {{
            futhark_entry_{entry_name}(self.context, {call_args})
        }};
        if rc != 0 {{ return Err(Error::Code(rc)); }}
    
        #[allow(unused_unsafe)]
        unsafe {{
            Ok({entry_return})
        }}
    }}
}}

extern "C" {{
    fn {entry_fn}(
        _: *mut futhark_context,
        {futhark_entry_params}
    ) -> core::ffi::c_int;
}}