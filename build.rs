// Copyright 2018-2019 the Deno authors. All rights reserved. MIT license.
use cargo_gn;
use std::env;
use std::path::Path;
use which::which;

fn main() {
  // On windows, rustc cannot link with a V8 debug build.
  let mut gn_args = if cargo_gn::is_debug() && !cfg!(target_os = "windows") {
    vec!["is_debug=true".to_string()]
  } else {
    vec!["is_debug=false".to_string()]
  };

  if let Some(clang_base_path) = env::var_os("CLANG_BASE_PATH") {
    gn_args.push(format!("clang_base_path={:?}", clang_base_path));
  } else if let Ok(p) = which("clang") {
    let bin = p.parent().unwrap();
    let clang_base_path = bin.parent().unwrap();
    gn_args.push(format!("clang_base_path={:?}", clang_base_path));
  } else {
    panic!("CLANG_BASE_PATH not set and 'clang' not found in path")
  }

  if let Some(p) = env::var_os("SCCACHE") {
    cc_wrapper(&mut gn_args, &Path::new(&p));
  } else if let Ok(p) = which("sccache") {
    cc_wrapper(&mut gn_args, &p);
  } else {
    println!("cargo:warning=Not using sccache");
  }

  // gn_root needs to be an absolute path.
  let gn_root = env::current_dir()
    .unwrap()
    .into_os_string()
    .into_string()
    .unwrap();

  let gn_out = cargo_gn::maybe_gen(&gn_root, gn_args);
  assert!(gn_out.exists());
  assert!(gn_out.join("args.gn").exists());
  cargo_gn::build("rusty_v8");

  println!("cargo:rustc-link-lib=static=rusty_v8");

  if cfg!(target_os = "windows") {
    println!("cargo:rustc-link-lib=dylib=winmm");
    println!("cargo:rustc-link-lib=dylib=dbghelp");
  }
}

fn cc_wrapper(gn_args: &mut Vec<String>, sccache_path: &Path) {
  gn_args.push(format!("cc_wrapper={:?}", sccache_path));

  // Disable treat_warnings_as_errors until this sccache bug is fixed:
  // https://github.com/mozilla/sccache/issues/264
  if cfg!(target_os = "windows") {
    gn_args.push("treat_warnings_as_errors=false".to_string());
  }
}
