use crate::*;
use std::io::Write;

/// Rust codegen
#[derive(Default)]
pub struct Rust;

fn is_primitive(a: &str, manifest: &Manifest) -> bool {
    !manifest.types.contains_key(a)
}
fn primitive_type_name(a: &str) -> &str {
    match a {
        "f16" => "half::f16",
        _ => a,
    }
}

impl Generate for Rust {
    fn array_type(
        &mut self,
        _pkg: &Package,
        config: &mut Config,
        name: &str,
        a: &manifest::ArrayType,
    ) -> Result<(), Error> {
        let elemtype = a.elemtype.to_str();

        let raw_type = config.raw_names[name].clone();
        let rust_type = config.type_names[name].clone();

        let dim_params = (0..a.rank)
            .map(|i| format!("dims[{i}] as i64"))
            .collect::<Vec<_>>()
            .join(", ");
        let new_dim_args = (0..a.rank)
            .map(|i| format!("dim{i}: i64"))
            .collect::<Vec<_>>()
            .join(", ");

        let index_params = (0..a.rank)
            .map(|i| format!("index[{i}] as i64"))
            .collect::<Vec<_>>()
            .join(", ");
        let index_args = (0..a.rank)
            .map(|i| format!("i{i}: i64"))
            .collect::<Vec<_>>()
            .join(", ");

        writeln!(
            config.output_file,
            include_str!("templates/rust/array.rs"),
            raw_type = raw_type,
            rust_type = rust_type,
            rank = a.rank,
            elemtype = elemtype,
            new_fn = a.ops.new,
            free_fn = a.ops.free,
            values_fn = a.ops.values,
            shape_fn = a.ops.shape,
            index_fn = a.ops.index,
            dim_params = dim_params,
            new_dim_args = new_dim_args,
            index_params = index_params,
            index_args = index_args,
        )?;

        Ok(())
    }

    fn opaque_type(
        &mut self,
        pkg: &Package,
        config: &mut Config,
        name: &str,
        ty: &manifest::OpaqueType,
    ) -> Result<(), Error> {
        let raw_type = config.raw_names[name].clone();
        let rust_type = config.type_names[name].clone();

        match &ty.options {
            manifest::OpaqueOptions::Record(record) => {
                writeln!(
                    config.output_file,
                    include_str!("templates/rust/opaque.rs"),
                    raw_type = raw_type,
                    rust_type = rust_type,
                    free_fn = ty.ops.free,
                    store_fn = ty.ops.store,
                    restore_fn = ty.ops.restore,
                )?;

                let mut new_call_args = vec![];
                let mut new_params = vec![];
                let mut new_extern_params = vec![];
                for field in record.fields.iter() {
                    // Build new function
                    let field_type = field.r#type.clone();
                    let prim = is_primitive(&field_type, &pkg.manifest);

                    let field_name = config.namer.new_field_name(&field.name, &pkg.manifest);
                    let rust_field_type = if prim {
                        primitive_type_name(&field_type).to_string()
                    } else {
                        config.type_names[&field_type].clone()
                    };
                    let raw_field_type = if prim {
                        rust_field_type.clone()
                    } else {
                        config.raw_names[&field_type].clone()
                    };

                    if prim {
                        new_call_args.push(field_name.clone());
                        new_extern_params.push(format!(
                            "f_{}: {}",
                            field.name,
                            primitive_type_name(&raw_field_type)
                        ));
                    } else {
                        new_call_args.push(format!("{}.ptr", field_name));
                        new_extern_params
                            .push(format!("f_{}: *const {}", field.name, raw_field_type));
                    }

                    new_params.push(format!("{}: {}", field_name, rust_field_type));

                    // Implement get function

                    // If the output type is an array or opaque type then we need to wrap the return value
                    let (output, raw_arg_type) = if is_primitive(&field_type, &pkg.manifest) {
                        ("Ok(out)".to_string(), raw_field_type)
                    } else {
                        (
                            format!("Ok({rust_field_type}::from_ptr(self.ctx, out))"),
                            format!("*mut {raw_field_type}"),
                        )
                    };

                    writeln!(
                        config.output_file,
                        include_str!("templates/rust/record_project.rs"),
                        project_fn = field.project,
                        rust_type = rust_type,
                        raw_type = raw_type,
                        field_name = field.name,
                        project_name = config.namer.project_name(&field.name, &pkg.manifest),
                        raw_arg_type = raw_arg_type,
                        rust_field_type = rust_field_type,
                        output = output
                    )?;
                }

                writeln!(
                    config.output_file,
                    include_str!("templates/rust/record.rs"),
                    rust_type = rust_type,
                    raw_type = raw_type,
                    new_fn = record.new,
                    new_params = new_params.join(", "),
                    new_call_args = new_call_args.join(", "),
                    new_extern_params = new_extern_params.join(", "),
                )?;
            }
            manifest::OpaqueOptions::Sum(_sum) => {
                writeln!(
                    config.output_file,
                    include_str!("templates/rust/opaque.rs"),
                    raw_type = raw_type,
                    rust_type = rust_type,
                    free_fn = ty.ops.free,
                    store_fn = ty.ops.store,
                    restore_fn = ty.ops.restore,
                )?;
            }
            manifest::OpaqueOptions::OpaqueArray(array)
            | manifest::OpaqueOptions::RecordArray(array) => {
                let rust_elemtype = config.type_names[&array.elemtype].clone();
                let raw_elemtype = config.raw_names[&array.elemtype].clone();

                let index_args = (0..array.rank)
                    .map(|i| format!("i{i}: i64"))
                    .collect::<Vec<_>>()
                    .join(", ");
                let index_params = (0..array.rank)
                    .map(|i| format!("index[{i}] as i64"))
                    .collect::<Vec<_>>()
                    .join(", ");

                writeln!(
                    config.output_file,
                    include_str!("templates/rust/opaque_array.rs"),
                    raw_type = raw_type,
                    rust_type = rust_type,
                    free_fn = ty.ops.free,
                    store_fn = ty.ops.store,
                    restore_fn = ty.ops.restore,
                    shape_fn = array.shape,
                    index_fn = array.index,
                    rank = array.rank,
                    rust_elemtype = rust_elemtype,
                    raw_elemtype = raw_elemtype,
                    index_args = index_args,
                    index_params = index_params,
                )?;

                let Some(record) = &array.record else {
                    return Ok(());
                };

                let mut zip_call_args = vec![];
                let mut zip_params = vec![];
                let mut zip_extern_params = vec![];

                for field in record.fields.iter() {
                    let field_type = field.r#type.clone();

                    let field_name = config.namer.new_field_name(&field.name, &pkg.manifest);
                    let rust_field_type = config.type_names[&field_type].clone();
                    let raw_field_type = config.raw_names[&field_type].clone();

                    zip_call_args.push(format!("{}.ptr", field_name));
                    zip_extern_params.push(format!("f_{}: *const {}", field.name, raw_field_type));
                    zip_params.push(format!("{}: &{}", field_name, rust_field_type));

                    let project_name = config.namer.project_name(&field.name, &pkg.manifest);
                    let project_fn = field.project.clone();

                    writeln!(
                        config.output_file,
                        include_str!("templates/rust/record_array_project.rs"),
                        project_fn = project_fn,
                        rust_type = rust_type,
                        raw_type = raw_type,
                        project_name = project_name,
                        raw_field_type = raw_field_type,
                        rust_field_type = rust_field_type,
                    )?;
                }

                writeln!(
                    config.output_file,
                    include_str!("templates/rust/record_array.rs"),
                    rust_type = rust_type,
                    raw_type = raw_type,
                    zip_params = zip_params.join(", "),
                    zip_call_args = zip_call_args.join(", "),
                    zip_extern_params = zip_extern_params.join(", "),
                    zip_fn = record.zip,
                )?;
            }
        }
        Ok(())
    }

    fn entry(
        &mut self,
        pkg: &Package,
        config: &mut Config,
        name: &str,
        entry: &manifest::Entry,
    ) -> Result<(), Error> {
        let mut call_args = Vec::new();
        let mut entry_params = Vec::new();
        let mut return_type = Vec::new();
        let mut out_decl = Vec::new();
        let mut futhark_entry_params = Vec::new();
        let mut entry_return = Vec::new();

        // Output arguments
        for (i, arg) in entry.outputs.iter().enumerate() {
            let futhark_type = arg.r#type.clone();
            let name = format!("out{i}");
            let prim = is_primitive(&futhark_type, &pkg.manifest);
            let raw_type = if prim {
                primitive_type_name(&futhark_type)
            } else {
                &config.raw_names[&futhark_type]
            };
            let rust_type = if prim {
                primitive_type_name(&futhark_type).to_string()
            } else {
                config.type_names[&futhark_type].to_string()
            };

            if prim {
                futhark_entry_params.push(format!("{name}: *mut {raw_type}"));
                entry_return.push(format!("{name}.assume_init()"));
                return_type.push(rust_type.clone());
            } else {
                futhark_entry_params.push(format!("{name}: *mut *mut {raw_type}"));
                entry_return.push(format!("{rust_type}::from_ptr(ctx, {name}.assume_init())",));
                return_type.push(format!("{rust_type}<'a>"));
            }

            out_decl.push(format!("let mut {name} = std::mem::MaybeUninit::zeroed();"));
            call_args.push(format!("{name}.as_mut_ptr()"));
        }

        // Input arguments
        for (i, arg) in entry.inputs.iter().enumerate() {
            let futhark_type = arg.r#type.clone();
            let name = format!("in{i}");

            let prim = is_primitive(&futhark_type, &pkg.manifest);

            let raw_type = if prim {
                primitive_type_name(&futhark_type)
            } else {
                &config.raw_names[&futhark_type]
            };
            let rust_type = if prim {
                primitive_type_name(&futhark_type).to_string()
            } else {
                config.type_names[&futhark_type].to_string()
            };

            if prim {
                futhark_entry_params.push(format!("{name}: {raw_type}"));
                entry_params.push(format!("{name}: {rust_type}"));
                call_args.push(name);
            } else {
                futhark_entry_params.push(format!("{name}: *const {raw_type}"));
                entry_params.push(format!("{name}: &{rust_type}<'a>"));
                call_args.push(format!("{name}.ptr as *mut _"));
            }
        }

        let (entry_return_type, entry_return) = match entry.outputs.len() {
            0 => ("()".to_string(), "()".to_string()),
            1 => (return_type.join(", "), entry_return.join(", ")),
            _ => (
                format!("({})", return_type.join(", ")),
                format!("({})", entry_return.join(", ")),
            ),
        };

        if config.entry_points_within_context {
            writeln!(
                config.output_file,
                include_str!("templates/rust/context_entry.rs"),
                entry_fn = entry.cfun,
                entry_name = name,
                entry_params = entry_params.join(", "),
                entry_return_type = entry_return_type,
                out_decl = out_decl.join(";\n"),
                call_args = call_args.join(", "),
                entry_return = entry_return,
                futhark_entry_params = futhark_entry_params.join(", "),
            )?;
        } else {
            writeln!(
                config.output_file,
                include_str!("templates/rust/entry.rs"),
                entry_fn = entry.cfun,
                entry_name = name,
                entry_params = entry_params.join(", "),
                entry_return_type = entry_return_type,
                out_decl = out_decl.join(";\n"),
                call_args = call_args.join(", "),
                entry_return = entry_return,
                futhark_entry_params = futhark_entry_params.join(", "),
            )?;
        }

        Ok(())
    }

    fn bindings(&mut self, pkg: &Package, config: &mut Config) -> Result<(), Error> {
        writeln!(config.output_file, "// Generated by futhark-bindgen\n")?;
        let backend_extern_functions = match &pkg.manifest.backend {
            Backend::Multicore => {
                "fn futhark_context_config_set_num_threads(_: *mut futhark_context_config, _: core::ffi::c_int);"
            }
            Backend::OpenCl | Backend::Cuda => {
                "fn futhark_context_config_set_device(_: *mut futhark_context_config, _: *const core::ffi::c_char);"
            }
            _ => "",
        };

        let backend_options = match pkg.manifest.backend {
            Backend::Multicore => {
                "pub fn threads(mut self, n: u32) -> Options { self.num_threads = n as u32; self }"
            }
            Backend::Cuda | Backend::OpenCl => {
                "pub fn device(mut self, s: impl AsRef<str>) -> Options { self.device = Some(std::ffi::CString::new(s.as_ref()).expect(\"Invalid device\")); self }"
            }
            _ => "",
        };

        let configure_num_threads = if pkg.manifest.backend == Backend::Multicore {
            "futhark_context_config_set_num_threads(config, options.num_threads as core::ffi::c_int);"
        } else {
            "let _ = &options.num_threads;"
        };

        let configure_set_device = if matches!(
            pkg.manifest.backend,
            Backend::Cuda | Backend::OpenCl
        ) {
            "if let Some(d) = &options.device { futhark_context_config_set_device(config, d.as_ptr()); }"
        } else {
            "let _ = &options.device;"
        };

        writeln!(
            config.output_file,
            include_str!("templates/rust/context.rs"),
            backend_options = backend_options,
            configure_num_threads = configure_num_threads,
            configure_set_device = configure_set_device,
            backend_extern_functions = backend_extern_functions,
        )?;

        Ok(())
    }

    fn format(&mut self, path: &std::path::Path) -> Result<(), Error> {
        let _ = std::process::Command::new("rustfmt").arg(path).status();
        Ok(())
    }
}
