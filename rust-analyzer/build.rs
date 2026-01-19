//! Build script for UE5 Fast Startup Accelerator
//! Compiles NASM assembly files for hot paths

use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=asm/");
    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let asm_dir = manifest_dir.join("asm");

    // Check if NASM is available
    let nasm_path = find_nasm();
    
    if let Some(nasm) = nasm_path {
        println!("cargo:warning=Found NASM at: {}", nasm.display());
        
        // Compile each ASM file
        let asm_files = ["hash_simd.asm", "memcpy_fast.asm", "scan_chunk.asm"];
        let mut obj_files = Vec::new();

        for asm_file in &asm_files {
            let asm_path = asm_dir.join(asm_file);
            if asm_path.exists() {
                let obj_name = asm_file.replace(".asm", ".obj");
                let obj_path = out_dir.join(&obj_name);

                println!("cargo:warning=Compiling {} -> {}", asm_file, obj_name);

                let status = Command::new(&nasm)
                    .args([
                        "-f", "win64",
                        "-o", obj_path.to_str().unwrap(),
                        asm_path.to_str().unwrap(),
                    ])
                    .status()
                    .expect("Failed to execute NASM");

                if status.success() {
                    obj_files.push(obj_path);
                    println!("cargo:warning=Successfully compiled {}", asm_file);
                } else {
                    println!("cargo:warning=Failed to compile {}", asm_file);
                }
            } else {
                println!("cargo:warning=ASM file not found: {}", asm_path.display());
            }
        }

        // Create static library from object files
        if !obj_files.is_empty() {
            let lib_path = out_dir.join("asm_hotpaths.lib");
            
            // Use llvm-ar or lib.exe to create static library
            let ar_result = create_static_lib(&obj_files, &lib_path);
            
            if ar_result {
                println!("cargo:rustc-link-search=native={}", out_dir.display());
                println!("cargo:rustc-link-lib=static=asm_hotpaths");
                println!("cargo:warning=Created static library: asm_hotpaths.lib");
            }
        }
    } else {
        println!("cargo:warning=NASM not found, using pure Rust fallback for hot paths");
    }
}

fn find_nasm() -> Option<PathBuf> {
    // Check common locations
    let possible_paths = [
        "nasm",
        "nasm.exe",
        r"C:\Users\andre\AppData\Local\bin\NASM\nasm.exe",
        r"C:\Program Files\NASM\nasm.exe",
        r"C:\Program Files (x86)\NASM\nasm.exe",
    ];

    for path in &possible_paths {
        let result = Command::new(path)
            .arg("--version")
            .output();

        if let Ok(output) = result {
            if output.status.success() {
                return Some(PathBuf::from(path));
            }
        }
    }

    // Check PATH
    if let Ok(path_var) = env::var("PATH") {
        for dir in path_var.split(';') {
            let nasm_path = PathBuf::from(dir).join("nasm.exe");
            if nasm_path.exists() {
                return Some(nasm_path);
            }
        }
    }

    None
}

fn create_static_lib(obj_files: &[PathBuf], lib_path: &PathBuf) -> bool {
    // Try llvm-ar first
    let llvm_ar = Command::new("llvm-ar")
        .arg("rcs")
        .arg(lib_path)
        .args(obj_files)
        .status();

    if let Ok(status) = llvm_ar {
        if status.success() {
            return true;
        }
    }

    // Try Microsoft lib.exe
    let lib_exe = Command::new("lib")
        .arg(format!("/OUT:{}", lib_path.display()))
        .args(obj_files)
        .status();

    if let Ok(status) = lib_exe {
        if status.success() {
            return true;
        }
    }

    // Try ar (MinGW)
    let ar = Command::new("ar")
        .arg("rcs")
        .arg(lib_path)
        .args(obj_files)
        .status();

    if let Ok(status) = ar {
        return status.success();
    }

    false
}
