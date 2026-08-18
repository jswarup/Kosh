//-- waveobjio.rs ------------------------------------------------------------------------------------------------------------------

use	std::fmt;
use	crate::{
    fenst::PtsPointsDto,
    fleck::{ Dir3f, Pt4f },
    flux::instream::{ FixedStream, IStream },
    shard::{ IGrammar, Parser, Charset },
    silo::{ Buff, IAccess, U32, U8 },
    ShardTree,
};

//---------------------------------------------------------------------------------------------------------------------------------

/// Represents a texture coordinate (u, v, w) in parameter space.
#[derive( Clone, Copy, Debug, PartialEq)]
pub struct TexCoord
{
    pub _U: f32,
    pub _V: f32,
    pub _W: f32,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl TexCoord
{
    pub fn	New( u: f32, v: f32) -> Self
    {
        Self {
            _U: u,
            _V: v,
            _W: 0.0,
        }
    }

    pub fn	WithW( u: f32, v: f32, w: f32) -> Self
    {
        Self {
            _U: u,
            _V: v,
            _W: w,
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Represents a single vertex reference within a polygonal face: v/vt/vn.
#[derive( Clone, Copy, Debug, PartialEq)]
pub struct FaceVertex
{
    pub _VertexIdx:   i32,
    pub _TexCoordIdx: Option< i32>,
    pub _NormalIdx:   Option< i32>,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl FaceVertex
{
    pub fn	New( vertexIdx: i32) -> Self
    {
        Self {
            _VertexIdx:   vertexIdx,
            _TexCoordIdx: None,
            _NormalIdx:   None,
        }
    }

    pub fn	WithTex( vertexIdx: i32, texCoordIdx: i32) -> Self
    {
        Self {
            _VertexIdx:   vertexIdx,
            _TexCoordIdx: Some( texCoordIdx),
            _NormalIdx:   None,
        }
    }

    pub fn	WithNormal( vertexIdx: i32, normalIdx: i32) -> Self
    {
        Self {
            _VertexIdx:   vertexIdx,
            _TexCoordIdx: None,
            _NormalIdx:   Some( normalIdx),
        }
    }

    pub fn	Full( vertexIdx: i32, texCoordIdx: i32, normalIdx: i32) -> Self
    {
        Self {
            _VertexIdx:   vertexIdx,
            _TexCoordIdx: Some( texCoordIdx),
            _NormalIdx:   Some( normalIdx),
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Represents a polygonal face containing a list of face vertices.
#[derive( Clone, Debug, PartialEq)]
pub struct Face
{
    pub _Vertices: Buff< FaceVertex>,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Face
{
    pub fn	New() -> Self
    {
        Self {
            _Vertices: Buff::New(),
        }
    }

    pub fn	Push( &mut self, vert: FaceVertex)
    {
        self._Vertices.Push( vert);
    }

    pub fn	Len( &self) -> usize
    {
        self._Vertices.Size().AsUsize()
    }

    pub fn	IsEmpty( &self) -> bool
    {
        self._Vertices.IsEmpty()
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Represents a parsed Wavefront .obj 3D model.
#[derive( Clone)]
pub struct WaveObjModel
{
    pub _Vertices:    Buff< Pt4f>,
    pub _TexCoords:   Buff< TexCoord>,
    pub _Normals:     Buff< Dir3f>,
    pub _Faces:       Buff< Face>,
    pub _Objects:     Buff< String>,
    pub _Groups:      Buff< String>,
    pub _MtlLibs:     Buff< String>,
    pub _UseMtls:     Buff< String>,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl WaveObjModel
{
    pub fn	New() -> Self
    {
        Self {
            _Vertices:    Buff::New(),
            _TexCoords:   Buff::New(),
            _Normals:     Buff::New(),
            _Faces:       Buff::New(),
            _Objects:     Buff::New(),
            _Groups:      Buff::New(),
            _MtlLibs:     Buff::New(),
            _UseMtls:     Buff::New(),
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	VertexCount( &self) -> U32
    {
        self._Vertices.Size()
    }

    pub fn	FaceCount( &self) -> U32
    {
        self._Faces.Size()
    }

    pub fn	NormalCount( &self) -> U32
    {
        self._Normals.Size()
    }

    pub fn	TexCoordCount( &self) -> U32
    {
        self._TexCoords.Size()
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	BoundingBox( &self) -> ( [f32; 3], [f32; 3])
    {
        if self._Vertices.IsEmpty() {
            return ( [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]);
        }
        let  	arr = self._Vertices.Arr();
        let  	first = arr.At( U32( 0));
        let  	mut minX = first._X;
        let  	mut minY = first._Y;
        let  	mut minZ = first._Z;
        let  	mut maxX = first._X;
        let  	mut maxY = first._Y;
        let  	mut maxZ = first._Z;
        for i in 1..self._Vertices.Size().AsUsize() {
            let  	v = arr.At( U32( i as u32));
            if v._X < minX { minX = v._X; }
            if v._Y < minY { minY = v._Y; }
            if v._Z < minZ { minZ = v._Z; }
            if v._X > maxX { maxX = v._X; }
            if v._Y > maxY { maxY = v._Y; }
            if v._Z > maxZ { maxZ = v._Z; }
        }
        ( [minX, minY, minZ], [maxX, maxY, maxZ])
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    /// Converts vertices to PtsPointsDto for seamless display with Fenst / Swarm.
    pub fn	ToDto( &self) -> PtsPointsDto
    {
        let  	( bboxMin, bboxMax) = self.BoundingBox();
        let  	totalPoints = self._Vertices.Size();
        let  	arr = self._Vertices.Arr();
        let  	pointsBuff = Buff::Create( totalPoints, |i| {
            let  	v = arr.At( i);
            [v._X, v._Y, v._Z]
        });
        PtsPointsDto {
            _Points:   pointsBuff,
            _Count:    totalPoints.AsUsize(),
            _BboxMin:  bboxMin,
            _BboxMax:  bboxMax,
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    /// Triangulates all polygon faces into triangle triples using fan triangulation.
    pub fn	Triangulate( &self) -> Buff< [FaceVertex; 3]>
    {
        let  	mut triangles = Buff::New();
        let  	facesArr = self._Faces.Arr();
        for fIdx in 0..self._Faces.Size().AsUsize() {
            let  	face = facesArr.At( U32( fIdx as u32));
            let  	vertCount = face.Len();
            if vertCount >= 3 {
                let  	vertsArr = face._Vertices.Arr();
                let  	v0 = *vertsArr.At( U32( 0));
                for i in 1..( vertCount - 1) {
                    let  	v1 = *vertsArr.At( U32( i as u32));
                    let  	v2 = *vertsArr.At( U32( ( i + 1) as u32));
                    triangles.Push( [v0, v1, v2]);
                }
            }
        }
        triangles
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

fn	ParseFaceVertexToken( token: &str, numVerts: usize, numTex: usize, numNorm: usize) -> Option< FaceVertex>
{
    let  	mut slashCount = 0;
    for b in token.bytes() {
        if b == b'/' {
            slashCount += 1;
        }
    }

    if slashCount == 0 {
        let  	vRaw: i32 = token.parse().ok()?;
        let  	vIdx = if vRaw < 0 { ( numVerts as i32) + vRaw + 1 } else { vRaw };
        return Some( FaceVertex::New( vIdx));
    }

    let  	parts: Vec< &str> = token.split( '/').collect();
    let  	vRaw: i32 = parts[0].parse().ok()?;
    let  	vIdx = if vRaw < 0 { ( numVerts as i32) + vRaw + 1 } else { vRaw };

    let  	vtIdx = if parts.len() > 1 && !parts[1].is_empty() {
        let  	vtRaw: i32 = parts[1].parse().ok()?;
        Some( if vtRaw < 0 { ( numTex as i32) + vtRaw + 1 } else { vtRaw } )
    } else {
        None
    };

    let  	vnIdx = if parts.len() > 2 && !parts[2].is_empty() {
        let  	vnRaw: i32 = parts[2].parse().ok()?;
        Some( if vnRaw < 0 { ( numNorm as i32) + vnRaw + 1 } else { vnRaw } )
    } else {
        None
    };

    Some( FaceVertex {
        _VertexIdx:   vIdx,
        _TexCoordIdx: vtIdx,
        _NormalIdx:   vnIdx,
    })
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Shard grammar struct that parses a Wavefront .obj stream into a `WaveObjModel`.
pub struct WaveObjShard< 'a>
{
    pub _Model: &'a mut WaveObjModel,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> IGrammar for WaveObjShard< 'a>
{
    fn	Match( &self, parser: &mut Parser) -> bool
    {
        let  	modelPtr = self._Model as *const WaveObjModel as *mut WaveObjModel;
        let  	model = unsafe { &mut *modelPtr };

        let  	mut m = parser.CurrMark();
        let  	nlGrammar = ShardTree!( ( ?'\r' < '\n' ) | '\r' );
        let  	skippable = ShardTree!( *(( "#" < *( Charset::EndLine().Negative()) < "\n") | [ " \r\n\t" ] ));

        while m < parser.InStream().Size() {
            if let Some( nextM) = parser.ParseGrammar( &skippable, m) {
                m = nextM;
            }

            if m >= parser.InStream().Size() {
                break;
            }

            // Find line start and identify command tag
            let  	lineStart = m;
            let  	mut lineEnd = m;
            while lineEnd < parser.InStream().Size() {
                let  	b = parser.GetAt( lineEnd);
                if b == U8( b'\r') || b == U8( b'\n') {
                    break;
                }
                if let Some( nextM) = parser.Incr( lineEnd) {
                    lineEnd = nextM;
                } else {
                    break;
                }
            }

            let  	lineBytes = parser.InStream().BytesAt( lineStart, lineEnd - lineStart);
            let  	lineStr = <&str>::from( lineBytes).trim();

            if !lineStr.is_empty() && !lineStr.starts_with( '#') {
                let  	mut tokens = lineStr.split_whitespace();
                if let Some( cmd) = tokens.next() {
                    match cmd {
                        "v" => {
                            let  	nums: Vec< f32> = tokens.filter_map( |t| t.parse::< f32>().ok()).collect();
                            if nums.len() >= 3 {
                                let  	w = if nums.len() >= 4 { nums[3] } else { 1.0 };
                                model._Vertices.Push( Pt4f::WithW( nums[0], nums[1], nums[2], w));
                            }
                        }
                        "vt" => {
                            let  	nums: Vec< f32> = tokens.filter_map( |t| t.parse::< f32>().ok()).collect();
                            if nums.len() >= 2 {
                                let  	w = if nums.len() >= 3 { nums[2] } else { 0.0 };
                                model._TexCoords.Push( TexCoord::WithW( nums[0], nums[1], w));
                            } else if nums.len() == 1 {
                                model._TexCoords.Push( TexCoord::New( nums[0], 0.0));
                            }
                        }
                        "vn" => {
                            let  	nums: Vec< f32> = tokens.filter_map( |t| t.parse::< f32>().ok()).collect();
                            if nums.len() >= 3 {
                                model._Normals.Push( Dir3f::New( nums[0], nums[1], nums[2]));
                            }
                        }
                        "f" => {
                            let  	numV = model._Vertices.Size().AsUsize();
                            let  	numT = model._TexCoords.Size().AsUsize();
                            let  	numN = model._Normals.Size().AsUsize();
                            let  	mut face = Face::New();
                            for token in tokens {
                                if let Some( fv) = ParseFaceVertexToken( token, numV, numT, numN) {
                                    face.Push( fv);
                                }
                            }
                            if !face.IsEmpty() {
                                model._Faces.Push( face);
                            }
                        }
                        "o" => {
                            if let Some( name) = tokens.next() {
                                model._Objects.Push( name.to_string());
                            }
                        }
                        "g" => {
                            if let Some( name) = tokens.next() {
                                model._Groups.Push( name.to_string());
                            }
                        }
                        "mtllib" => {
                            if let Some( name) = tokens.next() {
                                model._MtlLibs.Push( name.to_string());
                            }
                        }
                        "usemtl" => {
                            if let Some( name) = tokens.next() {
                                model._UseMtls.Push( name.to_string());
                            }
                        }
                        _ => {}
                    }
                }
            }

            m = lineEnd;
            if let Some( nextM) = parser.ParseGrammar( &nlGrammar, m) {
                m = nextM;
            }
        }

        parser.SetCurrMark( m);
        true
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Parses a Wavefront .obj 3D model from a string slice.
pub fn	ParseWaveObj( input: &str) -> Result< WaveObjModel, String>
{
    let  	mut stream = FixedStream::from( input);
    ParseWaveObjStream( &mut stream)
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Parses a Wavefront .obj 3D model from a raw byte slice.
pub fn	ParseWaveObjBytes( bytes: &[u8]) -> Result< WaveObjModel, String>
{
    let  	s = std::str::from_utf8( bytes).map_err( |e| e.to_string())?;
    ParseWaveObj( s)
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Parses a Wavefront .obj 3D model from an input stream.
pub fn	ParseWaveObjStream( stream: &mut dyn IStream) -> Result< WaveObjModel, String>
{
    let  	mut model = WaveObjModel::New();
    let  	mut parser = Parser::New( stream);
    let  	shard = WaveObjShard { _Model: &mut model };
    let  	res = parser.ParseGrammar( &shard, U32( 0));
    if res.is_some() {
        Ok( model)
    } else {
        Err( "Failed to parse Wavefront OBJ stream".to_string())
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl fmt::Debug for WaveObjModel
{
    fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result
    {
        f.debug_struct( "WaveObjModel")
            .field( "vertices", &self.VertexCount().AsUsize())
            .field( "tex_coords", &self.TexCoordCount().AsUsize())
            .field( "normals", &self.NormalCount().AsUsize())
            .field( "faces", &self.FaceCount().AsUsize())
            .finish()
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl fmt::Display for WaveObjModel
{
    fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result
    {
        write!( f, "WaveObjModel({} verts, {} faces)", self.VertexCount(), self.FaceCount())
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
