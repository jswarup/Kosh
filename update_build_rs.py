import os

with open('build.rs', 'r', encoding='utf-8') as f:
    content = f.read()

helper = '''
fn\tCompileWindowsManifest()
{
    if std::env::var( "CARGO_CFG_TARGET_OS").as_deref() != Ok( "windows") {
        return;
    }

    println!( "cargo:rerun-if-changed=res/kosh.manifest");
    println!( "cargo:rerun-if-changed=res/kosh.rc");

    let  \toutDir = std::env::var( "OUT_DIR").unwrap_or_default();
    let  \tresOutput = std::path::Path::new( &outDir).join( "kosh.res");

    let  \tmut rcPath: Option< std::path::PathBuf> = None;
    let  \tkitsBin = std::path::PathBuf::from( r"C:\").join( "Program Files (x86)").join( "Windows Kits").join( "10").join( "bin");
    if kitsBin.exists() {
        if let Ok( entries) = std::fs::read_dir( &kitsBin) {
            let  \tmut versions: Vec< _> = entries.flatten().map( |e| e.path()).collect();
            versions.sort();
            for v in versions.into_iter().rev() {
                let  \tcandidate = v.join( "x64").join( "rc.exe");
                if candidate.exists() {
                    rcPath = Some( candidate);
                    break;
                }
            }
        }
    }

    let  \trcExe = rcPath.unwrap_or_else( || std::path::PathBuf::from( "rc.exe"));
    let  \tstatus = std::process::Command::new( &rcExe)
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
'''

if 'CompileWindowsManifest' not in content:
    content = content.replace('fn\tmain()\n{', helper + '\nfn\tmain()\n{\n    CompileWindowsManifest();')
    with open('build.rs', 'w', encoding='utf-8', newline='\n') as f:
        f.write(content)
    print('Updated build.rs successfully.')
else:
    print('Already present.')
