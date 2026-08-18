//-- build.rs ----------------------------------------------------------------------------------------------------------------------
#![allow( non_snake_case)]

fn	main()
{
    // Compile the symph rust-gpu shader crate to SPIR-V at build time.
    // The resulting .spv path is emitted as a cargo env var for include_bytes!.
    let  	compileResult = spirv_builder::SpirvBuilder::new( "src/symph", "spirv-unknown-vulkan1.1")
        .build()
        .expect( "Failed to compile symph SPIR-V shader");

    let  	modulePath: std::path::PathBuf = match compileResult.module {
        spirv_builder::ModuleResult::SingleModule( p) => p,
        spirv_builder::ModuleResult::MultiModule( m) => m.into_iter().next().unwrap().1,
    };

    println!( "cargo::rustc-env=SYMPH_SPV_PATH={}", modulePath.display());

    // Sync tauri.conf.json from resource/ for Tauri build system
    let  	_ = std::fs::copy( "resource/tauri.conf.json", "tauri.conf.json");

    let  	attrs = tauri_build::Attributes::new()
        .capabilities_path_pattern( "resource/capabilities/*");
    tauri_build::try_build( attrs).expect( "Failed to build Tauri app");
}

//---------------------------------------------------------------------------------------------------------------------------------
