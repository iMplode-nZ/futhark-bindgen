use std::io::Write;

use crate::generate::first_uppercase;
use crate::*;

pub struct OCaml {
    typemap: BTreeMap<String, String>,
    ctypes_map: BTreeMap<String, String>,
    ba_map: BTreeMap<String, String>,
}

const OCAML_CTYPES_MAP: &[(&'static str, &'static str)] = &[
    ("i8", "int8_t"),
    ("u8", "uint8_t"),
    ("i16", "int16_t"),
    ("u16", "uint16_t"),
    ("i32", "int32_t"),
    ("u32", "uint32_t"),
    ("i64", "int64_t"),
    ("u64", "uint64_t"),
    ("f32", "float"),
    ("f64", "double"),
];

const OCAML_TYPE_MAP: &[(&'static str, &'static str)] = &[
    ("i8", "int"),
    ("u8", "int"),
    ("i16", "int"),
    ("u16", "int"),
    ("i32", "int32"),
    ("i64", "int64"),
    ("u32", "int32"),
    ("u64", "int64"),
    ("f32", "float"),
    ("f64", "float"),
];
const OCAML_BA_TYPE_MAP: &[(&'static str, &'static str)] = &[
    ("i8", "Bigarray.int8_signed_elt"),
    ("u8", "Bigarray.int8_unsigned_elt"),
    ("i16", "Bigarray.int16_signed_elt"),
    ("u16", "Bigarray.int16_unsigned_elt"),
    ("i32", "Bigarray.int32_elt"),
    ("i64", "Bigarray.int64_elt"),
    ("u32", "Bigarray.int32_elt"),
    ("u64", "Bigarray.int64_elt"),
    ("f32", "Bigarray.float32_elt"),
    ("f64", "Bigarray.float64_elt"),
];

fn type_is_array(t: &str) -> bool {
    t.contains("array_f") || t.contains("array_i") || t.contains("array_u") || t.contains("array_b")
}

fn type_is_opaque(t: &str) -> bool {
    t.contains(".t")
}

impl Default for OCaml {
    fn default() -> Self {
        let typemap = OCAML_TYPE_MAP
            .iter()
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .collect();

        let ba_map = OCAML_BA_TYPE_MAP
            .iter()
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .collect();

        let ctypes_map = OCAML_CTYPES_MAP
            .iter()
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .collect();

        OCaml {
            typemap,
            ba_map,
            ctypes_map,
        }
    }
}

impl OCaml {
    fn foreign_function(&mut self, name: &str, ret: &str, args: Vec<&str>) -> String {
        format!(
            "let {name} = Foreign.foreign \"{name}\" ({} @-> returning ({ret}))",
            args.join(" @-> ")
        )
    }

    fn get_ctype(&self, t: &str) -> String {
        self.ctypes_map
            .get(t)
            .cloned()
            .unwrap_or_else(|| t.to_string())
    }

    fn get_type(&self, t: &str) -> String {
        self.typemap
            .get(t)
            .cloned()
            .unwrap_or_else(|| t.to_string())
    }

    fn get_ba_type(&self, t: &str) -> String {
        self.ba_map.get(t).cloned().unwrap_or_else(|| t.to_string())
    }
}

impl Generate for OCaml {
    fn generate(&mut self, library: &Library, config: &mut Config) -> Result<(), Error> {
        let mut mli_file = std::fs::File::create(config.output_path.with_extension("mli"))?;

        macro_rules! ml {
            ($fmt:expr $(, $arg:expr)*$(,)?) => {
                writeln!(config.output_file, $fmt $(, $arg)*)?;
            };
            ($($fmt:expr $(, $arg:expr)*$(,)?);+$(;)?) => {
                $(
                    ml!($fmt $(, $arg)*);
                )+
            }
        }

        macro_rules! mli {
            ($fmt:expr $(, $arg:expr)*$(,)?) => {
                writeln!(mli_file, $fmt $(, $arg)*)?;
            };
            ($($fmt:expr $(, $arg:expr)*$(,)?);+$(;)?) => {
                $(
                    mli!($fmt $(, $arg)*);
                )+
            }
        }

        macro_rules! ml_no_newline {
            ($fmt:expr $(, $arg:expr)*$(,)?) => {
                write!(config.output_file, $fmt $(, $arg)*)?;
            };
            ($($fmt:expr $(, $arg:expr)*$(,)?);+$(;)?) => {
                $(
                    ml!($fmt $(, $arg)*);
                )+
            }
        }

        macro_rules! mli_no_newline {
           ($fmt:expr $(, $arg:expr)*$(,)?) => {
                write!(mli_file, $fmt $(, $arg)*)?;
            };
            ($($fmt:expr $(, $arg:expr)*$(,)?);+$(;)?) => {
                $(
                    mli!($fmt $(, $arg)*);
                )+
            }
        }

        mli!("(* Generated by futhark-bindgen *)\n");
        ml!("(* Generated by futhark-bindgen *)\n");

        ml!(
            "open Ctypes";
            "";
            "module Bindings = struct";
            "  external _stub: unit -> unit = \"futhark_context_new\"";
            "  let context = typedef (ptr void) \"context\"";
            "  let context_config = typedef (ptr void) \"context_config\"";
            "  {}", self.foreign_function("futhark_context_new", "context", vec!["context_config"]);
            "  {}", self.foreign_function("futhark_context_free", "int", vec!["context"]);
            "  {}", self.foreign_function("futhark_context_sync", "int", vec!["context"]);
            "  {}", self.foreign_function("futhark_context_config_new", "context_config", vec!["void"]);
            "  {}", self.foreign_function("futhark_context_config_free", "int", vec!["context"]);
            "  {}", self.foreign_function("futhark_context_config_set_profiling", "void", vec!["context_config", "int"]);
            "  {}", self.foreign_function("futhark_context_config_set_debugging", "void", vec!["context_config", "int"]);
            "  {}", self.foreign_function("futhark_context_config_set_logging", "void", vec!["context_config", "int"]);
            "  {}", self.foreign_function("futhark_context_config_set_cache_file", "void", vec!["context_config", "string"]);
            "  {}", self.foreign_function("futhark_context_pause_profiling", "void", vec!["context"]);
            "  {}", self.foreign_function("futhark_context_unpause_profiling", "void", vec!["context"]);
            "  {}", self.foreign_function("futhark_context_clear_caches", "int", vec!["context"]);
            "  {}", self.foreign_function("futhark_context_get_error", "ptr char", vec!["context"]);
            "  {}", self.foreign_function("futhark_context_report", "ptr char", vec!["context"]);
            "  {}", self.foreign_function("free", "void", vec!["ptr void"]);
            "  {}", self.foreign_function("strlen", "size_t", vec!["ptr char"]);
        );

        for (name, ty) in &library.manifest.types {
            match ty {
                manifest::Type::Array(a) => {
                    let elemtype = a.elemtype.to_str().to_string();
                    let ctypes_elemtype = self.get_ctype(&elemtype);
                    let rank = a.rank;
                    let ocaml_name = format!("array_{elemtype}_{rank}d");
                    self.typemap.insert(name.clone(), ocaml_name.clone());
                    self.ctypes_map.insert(name.clone(), ocaml_name.clone());
                    let elem_ptr = format!("ptr {ctypes_elemtype}");
                    ml!("  let {ocaml_name} = typedef (ptr void) \"array_{elemtype}_{rank}d\"");
                    let mut new_args = vec!["context", &elem_ptr];
                    for _ in 0..rank {
                        new_args.push("int64_t");
                    }
                    ml!(
                        "  {}",
                        self.foreign_function(
                            &format!("futhark_new_{elemtype}_{rank}d"),
                            &ocaml_name,
                            new_args
                        );

                        "  {}",
                        self.foreign_function(
                            &format!("futhark_values_{elemtype}_{rank}d"),
                            "int",
                            vec!["context", &ocaml_name, &elem_ptr]
                        );
                        "  {}",
                        self.foreign_function(
                            &format!("futhark_free_{elemtype}_{rank}d"),
                            "int",
                            vec!["context", &ocaml_name]
                        );
                        "  {}", self.foreign_function(
                            &format!("futhark_shape_{elemtype}_{rank}d"),
                            "ptr int64_t",
                            vec!["context", &ocaml_name]
                        );
                    );
                }
                manifest::Type::Opaque(ty) => {
                    let new_fn = &ty.record.new;
                    ml!("  let {name} = typedef (ptr void) \"{name}\"");
                    let mut args = vec!["context".to_string(), format!("ptr {name}")];
                    for f in ty.record.fields.iter() {
                        let cty = self
                            .ctypes_map
                            .get(&f.r#type)
                            .cloned()
                            .unwrap_or_else(|| f.r#type.clone());
                        args.push(cty);
                    }
                    let args = args.iter().map(|x| x.as_str()).collect();
                    ml!("  {}", self.foreign_function(new_fn, "int", args));

                    let free_fn = &ty.ops.free;
                    ml!(
                        "  {}",
                        self.foreign_function(free_fn, "int", vec!["context", name])
                    );

                    for f in ty.record.fields.iter() {
                        let cty = self
                            .ctypes_map
                            .get(&f.r#type)
                            .cloned()
                            .unwrap_or_else(|| f.r#type.clone());
                        ml!(
                            "  {}",
                            self.foreign_function(
                                &f.project,
                                "int",
                                vec!["context", &format!("ptr {cty}"), name]
                            )
                        );
                    }
                }
            }
        }

        for (_name, entry) in &library.manifest.entry_points {
            let mut args = vec!["context".to_string()];

            for out in &entry.outputs {
                let t = self.get_ctype(&out.r#type);

                /*if type_is_array(&t) || type_is_opaque(&t) {
                    args.push(t);
                } else {*/
                args.push(format!("ptr {t}"));
                //}
            }

            for input in &entry.inputs {
                let t = self.get_type(&input.r#type);
                args.push(t);
            }

            let args = args.iter().map(|x| x.as_str()).collect();
            ml!("  {}", self.foreign_function(&entry.cfun, "int", args));
        }

        ml!("end");

        let error_t = "type error = InvalidShape of int * int | NullPtr | Code of int\nexception Error of error";
        ml!(
            "{}", error_t;
            "let () = Printexc.register_printer (function \
            | Error (InvalidShape (a, b)) -> Some (Printf.sprintf \"futhark error: invalid shape, expected %d but got %d\" a b) \
            | Error NullPtr -> Some \"futhark error: null pointer\" \
            | Error (Code c) -> Some (Printf.sprintf \"futhark error: code %d\" c) | _ -> None)"
        );

        mli!("{}", error_t); // mli

        ml!(
            "open Bigarray";
            "";
            "module Context = struct";
            "  type t = {} handle: unit ptr; config: unit ptr; cache_file: string option {}", '{', '}';
            "";
            "  let free t = \
            ignore (Bindings.futhark_context_free t.handle); \
            ignore (Bindings.futhark_context_config_free t.config)";
            "";
            "  let v ?(debug = false) ?(log = false) ?(profile = false) ?cache_file () =";
            "    let config = Bindings.futhark_context_config_new () in";
            "    if is_null config then raise (Error NullPtr);";
            "    Bindings.futhark_context_config_set_debugging config (if debug then 1 else 0);";
            "    Bindings.futhark_context_config_set_profiling config (if profile then 1 else 0);";
            "    Bindings.futhark_context_config_set_logging config (if log then 1 else 0);";
            "    Option.iter (Bindings.futhark_context_config_set_cache_file config) cache_file;";
            "    let handle = Bindings.futhark_context_new config in";
            "    if is_null handle then (ignore @@ Bindings.futhark_context_config_free config; raise (Error NullPtr));";
            "    let t = {} handle; config; cache_file {} in", '{', '}';
            "    Gc.finalise free t; t";
            "";
            "  let sync t = let rc = Bindings.futhark_context_sync t.handle in if rc <> 0 then raise (Error (Code rc))";
            "";
            "  let clear_caches t = let rc = Bindings.futhark_context_clear_caches t.handle in if rc <> 0 then raise (Error (Code rc))";
            "";
            "  let string_opt_of_ptr ptr = ";
            "    if is_null ptr then None";
            "    else";
            "      let len = Bindings.strlen ptr |> Unsigned.Size_t.to_int in";
            "      let s = String.init len (fun i -> !@(ptr +@ i)) in";
            "      let () = Bindings.free (coerce (Ctypes.ptr Ctypes.char) (Ctypes.ptr void) ptr) in Some s";
            "";
            "  let get_error t = let ptr = Bindings.futhark_context_get_error t.handle in string_opt_of_ptr ptr";
            "";
            "  let report t = let ptr = Bindings.futhark_context_report t.handle in string_opt_of_ptr ptr";
            "";
            "  let pause_profiling t = Bindings.futhark_context_pause_profiling t.handle";
            "  let unpause_profiling t = Bindings.futhark_context_unpause_profiling t.handle";
            "end"
        );

        // mli
        mli!(
            "module Context: sig";
            "  type t";
            "  val v: ?debug:bool -> ?log:bool -> ?profile:bool -> ?cache_file:string -> unit -> t";
            "  val sync: t -> unit";
            "  val clear_caches: t -> unit";
            "  val get_error: t -> string option";
            "  val report: t -> string option";
            "  val pause_profiling: t -> unit";
            "  val unpause_profiling: t -> unit";
            "end");

        ml!(
            "type futhark_array = {} ptr: unit ptr; shape: int array; ctx: Context.t {}",
            '{',
            '}';
             "type opaque = {} opaque_ptr: unit ptr; opaque_ctx: Context.t {}",
            '{',
            '}';
        );

        for (name, ty) in &library.manifest.types {
            match ty {
                manifest::Type::Array(a) => {
                    let rank = a.rank;
                    let elemtype = a.elemtype.to_str().to_string();
                    let ocaml_name = self.typemap.get(name).unwrap();
                    let module_name = first_uppercase(&ocaml_name);
                    ml!(
                        "module {} = struct", &module_name;
                        "  type t = futhark_array";
                        "";
                        "  let free t = ignore (Bindings.futhark_free_{elemtype}_{rank}d t.ctx.Context.handle t.ptr)";
                    );
                    ml!(
                        "  let v ctx ba =";
                        "    let dims = Genarray.dims ba in";
                        "    let ptr = Bindings.futhark_new_{elemtype}_{rank}d ctx.Context.handle (bigarray_start genarray ba)";
                    );

                    for i in 0..rank {
                        ml_no_newline!(" (Int64.of_int dims.({i}))");
                    }

                    ml!(
                        " in";
                        "    if is_null ptr then raise (Error NullPtr);";
                        "    let t = {} ptr; ctx; shape = dims {} in", '{', '}';
                        "    Gc.finalise free t; t";
                        "";
                        "  let values t ba =";
                        "    let dims = Genarray.dims ba in";
                        "    let a = Array.fold_left ( * ) 1 t.shape in";
                        "    let b = Array.fold_left ( * ) 1 dims in";
                        "    if (a <> b) then raise (Error (InvalidShape (a, b)));";
                        "    let rc = Bindings.futhark_values_{elemtype}_{rank}d t.ctx.Context.handle t.ptr (bigarray_start genarray ba) in";
                        "    if rc <> 0 then raise (Error (Code rc))";
                        "";
                        "  let shape t = t.shape";
                        "";
                        "  let raw_shape ctx ptr = ";
                        "    let s = Bindings.futhark_shape_{elemtype}_{rank}d ctx ptr in";
                        "    Array.init {rank} (fun i -> Int64.to_int !@ (s +@ i))";
                        "";
                        "  let of_raw ctx ptr =";
                        "    if is_null ptr then raise (Error NullPtr);";
                        "    let shape = raw_shape ctx.Context.handle ptr in";
                        "    let t = {} ptr; ctx; shape {} in", '{', '}';
                        "    Gc.finalise free t; t";
                        "";
                        "  let _ = of_raw";
                        "end"
                    );

                    let ocaml_elemtype = self.get_type(&elemtype);
                    let ba_elemtype = self.get_ba_type(&elemtype);

                    // mli
                    mli!(
                        "module {module_name}: sig";
                        "  type t";
                        "  val shape: t -> int array";
                        "  val v: Context.t -> ({ocaml_elemtype}, {ba_elemtype}, Bigarray.c_layout) Bigarray.Genarray.t -> t";
                        "  val values: t -> ({ocaml_elemtype}, {ba_elemtype}, Bigarray.c_layout) Bigarray.Genarray.t -> unit";
                        "end"
                    );
                }
                manifest::Type::Opaque(ty) => {
                    let module_name = first_uppercase(name);
                    self.typemap
                        .insert(name.clone(), format!("{module_name}.t"));

                    ml!(
                        "module {module_name} = struct";
                        "  type t = opaque";
                        "  let t = Bindings.{name}";
                        "  let _ = t";
                    );

                    mli!(
                        "module {module_name} : sig";
                        "  type t";
                    );

                    let free_fn = &ty.ops.free;
                    ml!("  let free t = ignore (Bindings.{free_fn} t.opaque_ctx.Context.handle t.opaque_ptr)");
                    ml_no_newline!("  let v ctx");
                    mli_no_newline!("  val v: Context.t");

                    let mut args = vec![];
                    for f in ty.record.fields.iter() {
                        let t = self.get_type(&f.r#type);
                        ml_no_newline!(" field{}", f.name);

                        if type_is_array(&t) {
                            args.push(format!("field{}.ptr", f.name));

                            mli_no_newline!(" -> {}.t", first_uppercase(&t));
                        } else if type_is_opaque(&t) {
                            args.push(format!("field{}.opaque_ptr", f.name));

                            mli_no_newline!(" -> {t}");
                        } else {
                            args.push(format!("field{}", f.name));

                            mli_no_newline!(" -> {t}");
                        }
                    }

                    ml!(" = ");
                    mli!(" -> t");

                    let new_fn = &ty.record.new;
                    ml!(
                        "    let ptr = allocate (ptr void) null in";
                        "    let rc = Bindings.{new_fn} ctx.Context.handle ptr {} in", args.join(" ");
                        "    if rc <> 0 then raise (Error (Code rc));";
                        "    let opaque_ptr = !@ptr in";
                        "    let t = {} opaque_ptr; opaque_ctx = ctx {} in", '{', '}';
                        "    Gc.finalise free t; t";
                    );

                    ml!(
                        "  let of_raw ctx ptr =";
                        "    if is_null ptr then raise (Error NullPtr);";
                        "    let t = {} opaque_ptr = ptr; opaque_ctx = ctx {} in", '{', '}';
                        "    Gc.finalise free t; t";
                        "";
                        "let _ = of_raw";
                    );

                    for f in ty.record.fields.iter() {
                        let t = self.get_type(&f.r#type);
                        let name = &f.name;
                        let project = &f.project;

                        let s = if type_is_array(&t) {
                            format!("Bindings.{t}")
                        } else {
                            t.clone()
                        };

                        ml!(
                            "  let get_{name} t =";
                            "    let out = allocate_n ~count:1 {s} in";
                            "    let rc = Bindings.{project} t.opaque_ctx.Context.handle out t.opaque_ptr in";
                            "    if rc <> 0 then raise (Error (Code rc));";
                        );

                        if type_is_opaque(&t) {
                            ml!(
                                "    let t = {} opaque_ptr = !@out; opaque_ctx = t.opaque_ctx {} in", '{', '}';
                                "    Gc.finalise free t; t";
                            );
                        } else if type_is_array(&t) {
                            let array = first_uppercase(&t);
                            ml!(
                                "    let shape = {array}.raw_shape t.opaque_ctx.Context.handle !@out in";
                                "    let t = {} ptr = !@out; ctx = t.opaque_ctx; shape {} in", '{', '}';
                                "    Gc.finalise {array}.free t; t"
                            );
                        } else {
                            ml!("    !@out");
                        }

                        if type_is_array(&t) {
                            mli!("  val get_{name}: t -> {}.t", first_uppercase(&t));
                        } else {
                            mli!("  val get_{name}: t -> {t}");
                        }
                    }

                    ml!("end");
                    mli!("end");
                }
            }
        }

        ml!("module Entry = struct");
        mli!("module Entry: sig");
        for (name, entry) in &library.manifest.entry_points {
            ml_no_newline!("  let {} ctx", name);
            mli_no_newline!("  val {}: Context.t", name); // mli

            for (i, input) in entry.inputs.iter().enumerate() {
                ml_no_newline!(" input{i}");

                let mut ocaml_elemtype = self.get_type(&input.r#type);

                // Transform into `Module.t`
                if type_is_array(&ocaml_elemtype) {
                    ocaml_elemtype = first_uppercase(&ocaml_elemtype) + ".t"
                }

                mli_no_newline!(" -> {}", ocaml_elemtype); // mli
            }

            mli_no_newline!(" -> (");

            for (i, out) in entry.outputs.iter().enumerate() {
                let mut ocaml_elemtype = self.get_type(&out.r#type);

                // Transform into `Module.t`
                if ocaml_elemtype.contains("array_") {
                    ocaml_elemtype = first_uppercase(&ocaml_elemtype) + ".t"
                }

                if i == entry.outputs.len() - 1 {
                    mli_no_newline!("{ocaml_elemtype}"); // mli
                } else {
                    mli_no_newline!("{ocaml_elemtype},"); // mli
                }
            }
            mli!(")"); // mli
            ml!(" =");

            for (i, out) in entry.outputs.iter().enumerate() {
                let t = self.get_type(&out.r#type);
                let ct = self.get_ctype(&out.r#type);

                let i = if entry.outputs.len() == 1 {
                    String::new()
                } else {
                    format!("{i}")
                };

                if type_is_array(&t) {
                    ml!("    let out{i}_ptr = allocate_n (ptr void) ~count:1 in");
                } else if type_is_opaque(&t) {
                    ml!("    let out{i}_ptr = allocate_n (ptr void) ~count:1) in");
                } else {
                    ml!("    let out{i}_ptr = allocate_n {ct} ~count:1 in");
                }
            }

            ml_no_newline!(
                "    let rc = Bindings.futhark_entry_{name} ctx.Context.handle";
            );

            for (i, _out) in entry.outputs.iter().enumerate() {
                let i = if entry.outputs.len() == 1 {
                    String::new()
                } else {
                    format!("{i}")
                };
                ml_no_newline!(" out{i}_ptr");
            }

            for (i, input) in entry.inputs.iter().enumerate() {
                let t = self.get_type(&input.r#type);
                if type_is_array(&t) {
                    ml_no_newline!(" input{i}.ptr");
                } else if type_is_opaque(&t) {
                    ml_no_newline!(" input{i}.opaque_ptr");
                } else {
                    ml_no_newline!(" input{i}");
                }
            }
            ml!(
                " in";
                "    if rc <> 0 then raise (Error (Code rc));"
            );

            ml_no_newline!("(");
            for (i, out) in entry.outputs.iter().enumerate() {
                let t = self.get_type(&out.r#type);

                let idx = if entry.outputs.len() == 1 {
                    String::new()
                } else {
                    format!("{i}")
                };

                if type_is_array(&t) {
                    let m = first_uppercase(&t);
                    ml_no_newline!("({m}.of_raw ctx !@out{idx}_ptr)");
                } else if type_is_opaque(&t) {
                    let m = first_uppercase(&t);
                    ml_no_newline!("({m}.of_raw ctx !@out{idx}_ptr)");
                } else {
                    ml_no_newline!("!@out{idx}_ptr");
                }

                if i != entry.outputs.len() - 1 {
                    ml_no_newline!(", ");
                }
            }
            ml!(")");
        }
        ml!("end");
        mli!("end");

        Ok(())
    }
}
