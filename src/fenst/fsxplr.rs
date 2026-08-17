//-- fenst/fsxplr.rs ---------------------------------------------------------------------------------------------------------------
use	crate::fenst::xplr::{ Xplr, LeafXplr, BranchXplr };
use	crate::silo::{ Buff, U32 };
use	std::fs;
use	std::path::{ Path, PathBuf };

// ---------------------------------------------------------------------------------------------------------------------------------

pub struct FsLeaf
{
    name:       String,
    path:       String,
    extension:  String,
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
            name,
            path,
            extension,
        }
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------

impl Xplr for FsLeaf
{
    fn	Name( &self) -> &str
    {
        &self.name
    }

    fn	Path( &self) -> &str
    {
        &self.path
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
        fs::metadata( &self.path).map( |m| m.len()).unwrap_or( 0)
    }

    fn	Extension( &self) -> &str
    {
        &self.extension
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------

pub struct FsBranch
{
    name: String,
    path: String,
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
            name,
            path,
        }
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------

impl Xplr for FsBranch
{
    fn	Name( &self) -> &str
    {
        &self.name
    }

    fn	Path( &self) -> &str
    {
        &self.path
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
        let  	dirPath = Path::new( &self.path);
        if !dirPath.exists() {
            return Err( format!( "Path does not exist: {}", self.path));
        }
        if !dirPath.is_dir() {
            return Err( format!( "Path is not a directory: {}", self.path));
        }

        let  	readDir = fs::read_dir( dirPath)
            .map_err( |e| format!( "Failed to read directory: {}", e))?;

        let  	mut dirs: Buff< Box< dyn Xplr>> = Buff::NewEmpty();
        let  	mut files: Buff< Box< dyn Xplr>> = Buff::NewEmpty();

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
                dirs.Push( Box::new( FsBranch::New( pathStr)));
            } else {
                files.Push( Box::new( FsLeaf::New( pathStr)));
            }
        }

        dirs.sort_by( |a, b| a.Name().to_lowercase().cmp( &b.Name().to_lowercase()));
        files.sort_by( |a, b| a.Name().to_lowercase().cmp( &b.Name().to_lowercase()));

        let  	mut result = Buff::NewEmpty();
        for d in dirs {
            result.Push( d);
        }
        for f in files {
            result.Push( f);
        }
        Ok( result)
    }

    fn	ChildCount( &self) -> Result< U32, String>
    {
        let  	dirPath = Path::new( &self.path);
        if !dirPath.exists() {
            return Err( format!( "Path does not exist: {}", self.path));
        }
        if !dirPath.is_dir() {
            return Err( format!( "Path is not a directory: {}", self.path));
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
