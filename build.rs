//-- build.rs ----------------------------------------------------------------------------------------------------------------------
#![allow( non_snake_case)]

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

fn	main()
{
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

    // Sync tauri.conf.json from src/frieze/ for Tauri build system
    let  	_ = std::fs::copy( "src/frieze/tauri.conf.json", "tauri.conf.json");

    let  	attrs = tauri_build::Attributes::new()
        .capabilities_path_pattern( "src/frieze/capabilities/*");
    tauri_build::try_build( attrs).expect( "Failed to build Tauri app");

    // Relocate generated schemas to out/gen and clean root
    let  	genPath = std::path::Path::new( "gen");
    if genPath.exists() {
        let  	outGenPath = std::path::Path::new( "out/gen");
        CopyDirAll( genPath, outGenPath);
        let  	_ = std::fs::remove_dir_all( genPath);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
