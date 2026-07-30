//-- main.rs ----------------------------------------------------------------------------------------------------------------------
#![allow( non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![cfg_attr( not( debug_assertions), windows_subsystem = "windows")]

// ---------------------------------------------------------------------------------------------------------------------------------

fn	main()
{
    // Workaround for WebKitGTK/WSL graphics issues (libEGL warning, ZINK failed to choose pdev)
    unsafe { 
        std::env::set_var( "WEBKIT_DISABLE_DMABUF_RENDERER", "1"); 
        std::env::set_var( "LIBGL_ALWAYS_SOFTWARE", "1");
    }
    fenst_app::run()
}

// ---------------------------------------------------------------------------------------------------------------------------------
