//-- build.rs ----------------------------------------------------------------------------------------------------------------------
#![allow( non_snake_case, non_camel_case_types, non_upper_case_globals)]

fn	CopyDirAll( src: &std::path::Path, dst: &std::path::Path)
{
    let  	_ = std::fs::create_dir_all( dst);
    if let Ok( entries) = std::fs::read_dir( src) {
        for entry in entries.flatten() {
            let  	entryPath = entry.path();
            let  	destPath = dst.join( entry.file_name());
            if entryPath.is_dir() {
                CopyDirAll( &entryPath, &destPath);
            } else {
                let  	_ = std::fs::copy( &entryPath, &destPath);
            }
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------


fn	CompileWindowsManifest()
{
    if std::env::var( "CARGO_CFG_TARGET_OS").as_deref() != Ok( "windows") {
        return;
    }

    println!( "cargo:rerun-if-changed=res/kosh.manifest");
    println!( "cargo:rerun-if-changed=res/kosh.rc");

    let  	outDir = std::env::var( "OUT_DIR").unwrap_or_default();
    let  	resOutput = std::path::Path::new( &outDir).join( "kosh.res");

    let  	mut rcPath: Option< std::path::PathBuf> = None;
    let  	kitsBin = std::path::PathBuf::from( r"C:").join( "Program Files (x86)").join( "Windows Kits").join( "10").join( "bin");
    if kitsBin.exists() {
        if let Ok( entries) = std::fs::read_dir( &kitsBin) {
            let  	mut versions: Vec< _> = entries.flatten().map( |e| e.path()).collect();
            versions.sort();
            for v in versions.into_iter().rev() {
                let  	candidate = v.join( "x64").join( "rc.exe");
                if candidate.exists() {
                    rcPath = Some( candidate);
                    break;
                }
            }
        }
    }

    let  	rcExe = rcPath.unwrap_or_else( || std::path::PathBuf::from( "rc.exe"));
    let  	status = std::process::Command::new( &rcExe)
        .args( [
            "/i", "res",
            "/fo", &resOutput.to_string_lossy(),
            "res/kosh.rc",
        ])
        .status();

    if let Ok( s) = status {
        if s.success() {
            println!( "cargo:rustc-link-arg={}", resOutput.display());
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

fn	main()
{
    CompileWindowsManifest();
    // Compile the symph rust-gpu shader crate to SPIR-V at build time.
    // The resulting .spv path is emitted as a cargo env var for include_bytes!.
    let  	compileResult = spirv_builder::SpirvBuilder::new( "src/symph", "spirv-unknown-vulkan1.1")
        .build()
        .expect( "Failed to compile symph SPIR-V shader");

    let  	modulePath: std::path::PathBuf = match compileResult.module {
        spirv_builder::ModuleResult::SingleModule( p) => p,
        spirv_builder::ModuleResult::MultiModule( ref m) => {
            for (k, v) in m {
                println!("cargo::warning=SPIRV Module entry: {} => {}", k, v.display());
            }
            m.get("camera_transform_cs")
                .or_else(|| m.get("main_cs"))
                .or_else(|| m.get("compshade"))
                .or_else(|| m.values().next())
                .cloned()
                .unwrap()
        }
    };

    println!( "cargo::rustc-env=SYMPH_SPV_PATH={}", modulePath.display());

    // Relocate generated schemas to out/gen and clean root
    let  	genPath = std::path::Path::new( "gen");
    if genPath.exists() {
        let  	outGenPath = std::path::Path::new( "out/gen");
        CopyDirAll( genPath, outGenPath);
        let  	_ = std::fs::remove_dir_all( genPath);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
