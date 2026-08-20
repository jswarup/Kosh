//-- waveobjio.rs ------------------------------------------------------------------------------------------------------------------
use	std::fmt;
use	crate::{
    fenst::{ PtsPointsDto, WaveObjMeshDto },
    fleck::{ Dir3f, WPt2f, WPt3f },
    flux::instream::{ FixedStream, IStream },
    shard::{ IGrammar, Parser, Charset },
    silo::{ Buff, Stash, IAccess, U32, U8 },
    ShardTree,
};

//---------------------------------------------------------------------------------------------------------------------------------

/// Represents an index reference into vertex, texture coordinate, and normal buffers for a polygon face.
#[derive( Clone, Copy, Debug, PartialEq, Eq)]
pub struct FaceVertex
{
    pub _VertexIdx:   i32,
    pub _TexCoordIdx: Option< i32>,
    pub _NormalIdx:   Option< i32>,
}

impl Default for FaceVertex
{
    fn	default() -> Self
    {
        Self {
            _VertexIdx:   0,
            _TexCoordIdx: None,
            _NormalIdx:   None,
        }
    }
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

    pub fn	WithTexCoord( vertexIdx: i32, texCoordIdx: i32) -> Self
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

impl Default for Face
{
    fn	default() -> Self
    {
        Self::New()
    }
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
        let  	mut stash = Stash::WithCapacity( self._Vertices.Size() + U32( 1));
        let  	arr = self._Vertices.Arr();
        for i in 0..self._Vertices.Size().AsUsize() {
            stash.Push( *arr.At( U32( i as u32)));
        }
        stash.Push( vert);
        self._Vertices = stash.IntoBuff();
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
    pub _Vertices:    Buff< WPt3f>,
    pub _TexCoords:   Buff< WPt2f>,
    pub _Normals:     Buff< Dir3f>,
    pub _Faces:       Buff< Face>,
    pub _Objects:     Buff< String>,
    pub _Groups:      Buff< String>,
    pub _MtlLibs:     Buff< String>,
    pub _UseMtls:     Buff< String>,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Default for WaveObjModel
{
    fn	default() -> Self
    {
        Self::New()
    }
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

    /// Converts WaveObjModel to WaveObjMeshDto with vertices, triangulated indices, unique wireframe edges, and face normals.
    pub fn	ToMeshDto( &self) -> WaveObjMeshDto
    {
        let  	( bboxMin, bboxMax) = self.BoundingBox();
        let  	vertCount = self._Vertices.Size();
        let  	arr = self._Vertices.Arr();
        let  	pointsBuff = Buff::Create( vertCount, |i| {
            let  	v = arr.At( i);
            [v._X, v._Y, v._Z]
        });

        let  	numFaces = self._Faces.Size().AsUsize();
        let  	mut trianglesStash = Stash::< [u32; 3]>::WithCapacity( U32( (numFaces * 2).max( 16) as u32));
        let  	mut edgeSet = std::collections::HashSet::new();
        let  	facesArr = self._Faces.Arr();

        for fIdx in 0..numFaces {
            let  	face = facesArr.At( U32( fIdx as u32));
            let  	faceVertCount = face.Len();
            if faceVertCount >= 3 {
                let  	vertsArr = face._Vertices.Arr();
                let  	v0Raw = vertsArr.At( U32( 0))._VertexIdx;
                let  	v0 = if v0Raw > 0 { ( v0Raw - 1) as u32 } else { 0 };

                for i in 1..( faceVertCount - 1) {
                    let  	v1Raw = vertsArr.At( U32( i as u32))._VertexIdx;
                    let  	v2Raw = vertsArr.At( U32( ( i + 1) as u32))._VertexIdx;
                    let  	v1 = if v1Raw > 0 { ( v1Raw - 1) as u32 } else { 0 };
                    let  	v2 = if v2Raw > 0 { ( v2Raw - 1) as u32 } else { 0 };
                    trianglesStash.Push( [v0, v1, v2]);
                }

                // Collect polygon boundary edges for wireframe
                for i in 0..faceVertCount {
                    let  	nextI = ( i + 1) % faceVertCount;
                    let  	eaRaw = vertsArr.At( U32( i as u32))._VertexIdx;
                    let  	ebRaw = vertsArr.At( U32( nextI as u32))._VertexIdx;
                    let  	ea = if eaRaw > 0 { ( eaRaw - 1) as u32 } else { 0 };
                    let  	eb = if ebRaw > 0 { ( ebRaw - 1) as u32 } else { 0 };
                    let  	edge = if ea < eb { ( ea, eb) } else { ( eb, ea) };
                    edgeSet.insert( edge);
                }
            }
        }

        let  	trianglesBuff = trianglesStash.IntoBuff();

        let  	mut edgesStash = Stash::< [u32; 2]>::WithCapacity( U32( edgeSet.len().max( 16) as u32));
        for ( e0, e1) in edgeSet {
            edgesStash.Push( [e0, e1]);
        }
        let  	edgesBuff = edgesStash.IntoBuff();

        // Compute per-triangle face normals for lighting
        let  	triArr = trianglesBuff.Arr();
        let  	numTriangles = trianglesBuff.Size();
        let  	normalsBuff = Buff::Create( numTriangles, |i| {
            let  	tri = triArr.At( i);
            let  	p0Idx = tri[0] as usize;
            let  	p1Idx = tri[1] as usize;
            let  	p2Idx = tri[2] as usize;
            if p0Idx < vertCount.AsUsize() && p1Idx < vertCount.AsUsize() && p2Idx < vertCount.AsUsize() {
                let  	p0 = arr.At( U32( p0Idx as u32));
                let  	p1 = arr.At( U32( p1Idx as u32));
                let  	p2 = arr.At( U32( p2Idx as u32));
                let  	ux = p1._X - p0._X;
                let  	uy = p1._Y - p0._Y;
                let  	uz = p1._Z - p0._Z;
                let  	vx = p2._X - p0._X;
                let  	vy = p2._Y - p0._Y;
                let  	vz = p2._Z - p0._Z;
                let  	nx = uy * vz - uz * vy;
                let  	ny = uz * vx - ux * vz;
                let  	nz = ux * vy - uy * vx;
                let  	len = ( nx * nx + ny * ny + nz * nz).sqrt();
                if len > 1e-6 {
                    [nx / len, ny / len, nz / len]
                } else {
                    [0.0, 1.0, 0.0]
                }
            } else {
                [0.0, 1.0, 0.0]
            }
        });

        WaveObjMeshDto {
            _Points:        pointsBuff,
            _Triangles:     trianglesBuff,
            _Edges:         edgesBuff,
            _Normals:       normalsBuff,
            _VertexCount:   vertCount.AsUsize(),
            _FaceCount:     numFaces,
            _BboxMin:       bboxMin,
            _BboxMax:       bboxMax,
        }
    }

    pub fn	Triangulate( &self) -> Buff< [FaceVertex; 3]>
    {
        let  	numFaces = self._Faces.Size().AsUsize();
        let  	mut trianglesStash = Stash::< [FaceVertex; 3]>::WithCapacity( U32( (numFaces * 2).max( 16) as u32));
        let  	facesArr = self._Faces.Arr();
        for fIdx in 0..numFaces {
            let  	face = facesArr.At( U32( fIdx as u32));
            let  	vertCount = face.Len();
            if vertCount >= 3 {
                let  	vertsArr = face._Vertices.Arr();
                let  	v0 = *vertsArr.At( U32( 0));
                for i in 1..( vertCount - 1) {
                    let  	v1 = *vertsArr.At( U32( i as u32));
                    let  	v2 = *vertsArr.At( U32( ( i + 1) as u32));
                    trianglesStash.Push( [v0, v1, v2]);
                }
            }
        }
        trianglesStash.IntoBuff()
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

/// Shard grammar struct that parses a Wavefront .obj stream into a WaveObjModel using Stash.
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

        let  	streamSz = parser.InStream().Size().AsUsize();
        let  	estVerts = (streamSz / 40).max( 128) as u32;
        let  	estFaces = (streamSz / 50).max( 128) as u32;

        let  	mut verticesStash = Stash::< WPt3f>::WithCapacity( U32( estVerts));
        let  	mut texCoordsStash = Stash::< WPt2f>::WithCapacity( U32( (estVerts / 2).max( 64)));
        let  	mut normalsStash = Stash::< Dir3f>::WithCapacity( U32( (estVerts / 2).max( 64)));
        let  	mut facesStash = Stash::< Face>::WithCapacity( U32( estFaces));
        let  	mut objectsStash = Stash::< String>::New();
        let  	mut groupsStash = Stash::< String>::New();
        let  	mut mtlLibsStash = Stash::< String>::New();
        let  	mut useMtlsStash = Stash::< String>::New();

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
                                verticesStash.Push( WPt3f::WithW( nums[0], nums[1], nums[2], w));
                            }
                        }
                        "vt" => {
                            let  	nums: Vec< f32> = tokens.filter_map( |t| t.parse::< f32>().ok()).collect();
                            if nums.len() >= 2 {
                                let  	w = if nums.len() >= 3 { nums[2] } else { 0.0 };
                                texCoordsStash.Push( WPt2f::WithW( nums[0], nums[1], w));
                            } else if nums.len() == 1 {
                                texCoordsStash.Push( WPt2f::New( nums[0], 0.0));
                            }
                        }
                        "vn" => {
                            let  	nums: Vec< f32> = tokens.filter_map( |t| t.parse::< f32>().ok()).collect();
                            if nums.len() >= 3 {
                                normalsStash.Push( Dir3f::New( nums[0], nums[1], nums[2]));
                            }
                        }
                        "f" => {
                            let  	numV = verticesStash.Size().AsUsize();
                            let  	numT = texCoordsStash.Size().AsUsize();
                            let  	numN = normalsStash.Size().AsUsize();
                            let  	mut faceVerts = Stash::< FaceVertex>::WithCapacity( U32( 4));
                            for token in tokens {
                                if let Some( fv) = ParseFaceVertexToken( token, numV, numT, numN) {
                                    faceVerts.Push( fv);
                                }
                            }
                            if faceVerts.Size() > U32( 0) {
                                facesStash.PushVal( Face { _Vertices: faceVerts.IntoBuff() });
                            }
                        }
                        "o" => {
                            if let Some( name) = tokens.next() {
                                objectsStash.PushVal( name.to_string());
                            }
                        }
                        "g" => {
                            if let Some( name) = tokens.next() {
                                groupsStash.PushVal( name.to_string());
                            }
                        }
                        "mtllib" => {
                            if let Some( name) = tokens.next() {
                                mtlLibsStash.PushVal( name.to_string());
                            }
                        }
                        "usemtl" => {
                            if let Some( name) = tokens.next() {
                                useMtlsStash.PushVal( name.to_string());
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

        model._Vertices = verticesStash.IntoBuff();
        model._TexCoords = texCoordsStash.IntoBuff();
        model._Normals = normalsStash.IntoBuff();
        model._Faces = facesStash.IntoBuff();
        model._Objects = objectsStash.IntoBuff();
        model._Groups = groupsStash.IntoBuff();
        model._MtlLibs = mtlLibsStash.IntoBuff();
        model._UseMtls = useMtlsStash.IntoBuff();

        parser.SetCurrMark( m);
        true
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Parses a Wavefront .obj file from a string slice.
pub fn	ParseWaveObj( input: &str) -> Result< WaveObjModel, String>
{
    let  	mut stream = FixedStream::from( input);
    ParseWaveObjStream( &mut stream)
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Parses a Wavefront .obj file from a raw byte slice.
pub fn	ParseWaveObjBytes( bytes: &[u8]) -> Result< WaveObjModel, String>
{
    let  	s = std::str::from_utf8( bytes).map_err( |e| e.to_string())?;
    ParseWaveObj( s)
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Parses a Wavefront .obj file from an input stream.
pub fn	ParseWaveObjStream( stream: &mut dyn IStream) -> Result< WaveObjModel, String>
{
    let  	mut model = WaveObjModel::New();
    let  	mut parser = Parser::New( stream);
    let  	shard = WaveObjShard { _Model: &mut model };
    let  	res = parser.ParseGrammar( &shard, U32( 0));
    if res.is_some() {
        Ok( model)
    } else {
        Err( "Failed to parse Wavefront .obj stream".to_string())
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
            .field( "objects", &self._Objects.Size().AsUsize())
            .field( "groups", &self._Groups.Size().AsUsize())
            .finish()
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl fmt::Display for WaveObjModel
{
    fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result
    {
        write!( f, "WaveObjModel(v: {}, f: {}, n: {})", self.VertexCount(), self.FaceCount(), self.NormalCount())
    }
}

//---------------------------------------------------------------------------------------------------------------------------------