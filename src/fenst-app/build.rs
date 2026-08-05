#![allow( non_snake_case)]
fn	main()
{
    // Compile the gcomp rust-gpu shader crate to SPIR-V at build time.
    // The resulting .spv path is emitted as a cargo env var for include_bytes!.
    let  	compileResult = spirv_builder::SpirvBuilder::new( "../../src/gcomp", "spirv-unknown-vulkan1.1")
        .build()
        .expect( "Failed to compile gcomp SPIR-V shader");

    let  	modulePath: std::path::PathBuf = match compileResult.module {
        spirv_builder::ModuleResult::SingleModule( p) => p,
        spirv_builder::ModuleResult::MultiModule( m) => m.into_iter().next().unwrap().1,
    };

    println!( "cargo::rustc-env=GCOMP_SPV_PATH={}", modulePath.display());

    tauri_build::build()
}
