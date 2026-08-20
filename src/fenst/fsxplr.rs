//-- fenst/fsxplr.rs ---------------------------------------------------------------------------------------------------------------
use	crate::fenst::xplr::{ Xplr, LeafXplr, BranchXplr };
use	crate::silo::{ Buff, U32 };
use	std::fs;
use	std::path::{ Path, PathBuf };

// ---------------------------------------------------------------------------------------------------------------------------------

pub struct FsLeaf
{
    _Name:       String,
    _Path:       String,
    _Extension:  String,
}

// ---------------------------------------------------------------------------------------------------------------------------------

impl FsLeaf
{
    pub fn	New( path: String) -> Self
    {
        let  	filePath = PathBuf::from( &path);
        let  	name = filePath.file_name()
            .map( |n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let  	extension = filePath.extension()
            .map( |e| e.to_string_lossy().into_owned())
            .unwrap_or_default();
        Self {
            _Name: name,
            _Path: path,
            _Extension: extension,
        }
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------

impl Xplr for FsLeaf
{
    fn	Name( &self) -> &str
    {
        &self._Name
    }

    fn	Path( &self) -> &str
    {
        &self._Path
    }

    fn	IsLeaf( &self) -> bool
    {
        true
    }

    fn	AsLeaf( &self) -> Option< &dyn LeafXplr>
    {
        Some( self)
    }

    fn	AsBranch( &self) -> Option< &dyn BranchXplr>
    {
        None
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------

impl LeafXplr for FsLeaf
{
    fn	Size( &self) -> u64
    {
        fs::metadata( &self._Path).map( |m| m.len()).unwrap_or( 0)
    }

    fn	Extension( &self) -> &str
    {
        &self._Extension
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------

pub struct FsBranch
{
    _Name: String,
    _Path: String,
}

// ---------------------------------------------------------------------------------------------------------------------------------

impl FsBranch
{
    pub fn	New( path: String) -> Self
    {
        let  	dirPath = PathBuf::from( &path);
        let  	name = dirPath.file_name()
            .map( |n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        Self {
            _Name: name,
            _Path: path,
        }
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------

impl Xplr for FsBranch
{
    fn	Name( &self) -> &str
    {
        &self._Name
    }

    fn	Path( &self) -> &str
    {
        &self._Path
    }

    fn	IsLeaf( &self) -> bool
    {
        false
    }

    fn	AsLeaf( &self) -> Option< &dyn LeafXplr>
    {
        None
    }

    fn	AsBranch( &self) -> Option< &dyn BranchXplr>
    {
        Some( self)
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------

impl BranchXplr for FsBranch
{
    fn	Children( &self) -> Result< Buff< Box< dyn Xplr>>, String>
    {
        let  	dirPath = Path::new( &self._Path);
        if !dirPath.exists() {
            return Err( format!( "Path does not exist: {}", self._Path));
        }
        if !dirPath.is_dir() {
            return Err( format!( "Path is not a directory: {}", self._Path));
        }

        let  	readDir = fs::read_dir( dirPath)
            .map_err( |e| format!( "Failed to read directory: {}", e))?;

        let  	mut dirs: Vec< Box< dyn Xplr>> = Vec::new();
        let  	mut files: Vec< Box< dyn Xplr>> = Vec::new();

        for entry in readDir {
            let  	entry = match entry {
                Ok( e) => e,
                Err( _) => continue,
            };
            let  	filePath = entry.path();
            let  	fileName = entry.file_name().to_string_lossy().into_owned();

            if fileName.starts_with( '.') {
                continue;
            }

            let  	metadata = match entry.metadata() {
                Ok( m) => m,
                Err( _) => continue,
            };

            let  	pathStr = filePath.to_string_lossy().into_owned();
            if metadata.is_dir() {
                dirs.push( Box::new( FsBranch::New( pathStr)));
            } else {
                files.push( Box::new( FsLeaf::New( pathStr)));
            }
        }

        dirs.sort_by( |a, b| a.Name().to_lowercase().cmp( &b.Name().to_lowercase()));
        files.sort_by( |a, b| a.Name().to_lowercase().cmp( &b.Name().to_lowercase()));

        let  	combined: Vec< Box< dyn Xplr>> = dirs.into_iter().chain( files.into_iter()).collect();
        Ok( Buff::FromVec( combined))
    }

    fn	ChildCount( &self) -> Result< U32, String>
    {
        let  	dirPath = Path::new( &self._Path);
        if !dirPath.exists() {
            return Err( format!( "Path does not exist: {}", self._Path));
        }
        if !dirPath.is_dir() {
            return Err( format!( "Path is not a directory: {}", self._Path));
        }

        let  	readDir = fs::read_dir( dirPath)
            .map_err( |e| format!( "Failed to read directory: {}", e))?;

        let  	mut count = 0;
        for entry in readDir {
            let  	entry = match entry {
                Ok( e) => e,
                Err( _) => continue,
            };
            let  	fileName = entry.file_name().to_string_lossy().into_owned();
            if fileName.starts_with( '.') {
                continue;
            }
            count += 1;
        }

        Ok( U32( count as u32))
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------
