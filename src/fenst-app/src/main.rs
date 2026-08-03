//-- main.rs ----------------------------------------------------------------------------------------------------------------------
#![allow( non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![cfg_attr( not( debug_assertions), windows_subsystem = "windows")]

// ---------------------------------------------------------------------------------------------------------------------------------

fn	main()
{
    // Workaround for WebKitGTK/WSL graphics issues:
    //   - WEBKIT_DISABLE_DMABUF_RENDERER:          suppresses libEGL/ZINK dma-buf failures
    //   - LIBGL_ALWAYS_SOFTWARE:                   force Mesa software renderer (no GPU needed)
    //   - WEBKIT_DISABLE_SANDBOX_THIS_IS_DANGEROUS: disable sandbox that blocks WebView in WSL
    //   NOTE: Do NOT set GDK_BACKEND=x11 — XWayland path causes 32x32 geometry bug in WSLg.
    //   NOTE: Do NOT set WEBKIT_DISABLE_COMPOSITING_MODE — it hides the WebView content.
    unsafe {
        std::env::set_var( "WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        std::env::set_var( "LIBGL_ALWAYS_SOFTWARE", "1");
        std::env::set_var( "WEBKIT_DISABLE_SANDBOX_THIS_IS_DANGEROUS", "1");
    }
    fenst_app::run()
}

// ---------------------------------------------------------------------------------------------------------------------------------
