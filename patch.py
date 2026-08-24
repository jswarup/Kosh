import sys

path = 'src/main.rs'
with open(path, 'r', encoding='utf-8') as f:
    content = f.read()

target1 = '''    /// Enable output prints from tests (nocapture)
    #[arg( short = 'g', long = "nocapture")]
    _Nocapture: bool,'''
replacement1 = '''    /// Enable output prints from tests (nocapture)
    #[arg( short = 'g', long = "nocapture")]
    _Nocapture: bool,
    /// Run native 3D GPU workspace
    #[arg( long = "frieze")]
    _Frieze: bool,'''
content = content.replace(target1, replacement1)

target2 = '''    // Default primary: 100% Native Pure-Rust frieze (wxDragon/wxWidgets + wgpu)
    if let  \tErr( e) = kosh::frieze::run() {
        eprintln!( "Error launching Kosh native window: {:?}", e);
    }'''
replacement2 = '''    // Default primary: 100% Native Pure-Rust frieze (wxDragon/wxWidgets + wgpu)
    if args._Frieze {
        if let  \tErr( e) = kosh::frieze::run() {
            eprintln!( "Error launching Kosh native window: {:?}", e);
        }
    }'''
content = content.replace(target2, replacement2)

with open(path, 'w', encoding='utf-8') as f:
    f.write(content)
print('Patch applied successfully')
