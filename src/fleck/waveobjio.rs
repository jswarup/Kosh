//-- waveobjio.rs ------------------------------------------------------------------------------------------------------------------
use	std::fmt;
use	crate::{
    fenst::{ PtsPointsDto, WaveObjMeshDto },
    fleck::{ BBox3f, Dir3f, Pt3f, WPt2f, WPt3f },
    flux::instream::{ FixedStream, IStream },
    shard::{ Charset, IGrammar, Int, Parser, Real },
    silo::{ Arr, U8, Buff, IAccess, Stash, U32 },
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
            stash.Push( *arr.At( i as u32));
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
        let  	mut bbox = BBox3f::Empty();
        for i in 0..self._Vertices.Size().AsUsize() {
            let  	v = arr.At( i as u32);
            bbox.Extend( Pt3f::New( v._X, v._Y, v._Z));
        }
        ( bbox.Min(), bbox.Max())
    }

    pub fn	BBox( &self) -> BBox3f
    {
        let  	( bboxMin, bboxMax) = self.BoundingBox();
        BBox3f::New( Pt3f::from( bboxMin), Pt3f::from( bboxMax))
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
            let  	face = facesArr.At( fIdx as u32);
            let  	faceVertCount = face.Len();
            if faceVertCount >= 3 {
                let  	vertsArr = face._Vertices.Arr();
                let  	v0Raw = vertsArr.At( 0)._VertexIdx;
                let  	v0 = if v0Raw > 0 { ( v0Raw - 1) as u32 } else { 0 };

                for i in 1..( faceVertCount - 1) {
                    let  	v1Raw = vertsArr.At( i as u32)._VertexIdx;
                    let  	v2Raw = vertsArr.At( ( i + 1) as u32)._VertexIdx;
                    let  	v1 = if v1Raw > 0 { ( v1Raw - 1) as u32 } else { 0 };
                    let  	v2 = if v2Raw > 0 { ( v2Raw - 1) as u32 } else { 0 };
                    trianglesStash.Push( [v0, v1, v2]);
                }

                // Collect polygon boundary edges for wireframe
                for i in 0..faceVertCount {
                    let  	nextI = ( i + 1) % faceVertCount;
                    let  	eaRaw = vertsArr.At( i as u32)._VertexIdx;
                    let  	ebRaw = vertsArr.At( nextI as u32)._VertexIdx;
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
                let  	p0 = arr.At( p0Idx as u32);
                let  	p1 = arr.At( p1Idx as u32);
                let  	p2 = arr.At( p2Idx as u32);
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
            let  	face = facesArr.At( fIdx as u32);
            let  	vertCount = face.Len();
            if vertCount >= 3 {
                let  	vertsArr = face._Vertices.Arr();
                let  	v0 = *vertsArr.At( 0);
                for i in 1..( vertCount - 1) {
                    let  	v1 = *vertsArr.At( i as u32);
                    let  	v2 = *vertsArr.At( ( i + 1) as u32);
                    trianglesStash.Push( [v0, v1, v2]);
                }
            }
        }
        trianglesStash.IntoBuff()
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
//---------------------------------------------------------------------------------------------------------------------------------

struct WaveObjParserCtx {
    _VStash: Stash< WPt3f>,
    _VtStash: Stash< WPt2f>,
    _VnStash: Stash< Dir3f>,
    _FStash: Stash< Face>,
    _ObjStash: Stash< String>,
    _GrpStash: Stash< String>,
    _MtlStash: Stash< String>,
    _UseMtlStash: Stash< String>,
    _Vals: Stash< f32>,
    _FaceVerts: Stash< FaceVertex>,
    _CurFaceVert: FaceVertex,
}

impl WaveObjParserCtx {
    fn	New( streamSz: U32) -> Self {
        let  	estVerts = (streamSz.0 / 40).max( 128);
        let  	estFaces = (streamSz.0 / 50).max( 128);
        Self {
            _VStash: Stash::< WPt3f>::WithCapacity( U32( estVerts)),
            _VtStash: Stash::< WPt2f>::WithCapacity( U32( (estVerts / 2).max( 64))),
            _VnStash: Stash::< Dir3f>::WithCapacity( U32( (estVerts / 2).max( 64))),
            _FStash: Stash::< Face>::WithCapacity( U32( estFaces)),
            _ObjStash: Stash::< String>::New(),
            _GrpStash: Stash::< String>::New(),
            _MtlStash: Stash::< String>::New(),
            _UseMtlStash: Stash::< String>::New(),
            _Vals: Stash::< f32>::WithCapacity( U32( 4)),
            _FaceVerts: Stash::< FaceVertex>::WithCapacity( U32( 4)),
            _CurFaceVert: FaceVertex { _VertexIdx: 0, _TexCoordIdx: None, _NormalIdx: None },
        }
    }
}

#[derive( Clone, Copy)]
struct WaveObjParserCtxMM( *mut WaveObjParserCtx);

impl WaveObjParserCtxMM {
    #[inline( always)]
    #[allow( clippy::mut_from_ref)]
    fn	Get( &self) -> &mut WaveObjParserCtx { unsafe { &mut *self.0 } }

    #[inline( always)]
    fn	PushMtlLib( &self, arr: Arr< '_, U8>) -> bool { self.Get()._MtlStash.Push( arr.AsStr().to_string()); true }
    
    #[inline( always)]
    fn	PushUseMtl( &self, arr: Arr< '_, U8>) -> bool { self.Get()._UseMtlStash.Push( arr.AsStr().to_string()); true }
    
    #[inline( always)]
    fn	PushVal( &self, arr: Arr< '_, U8>) -> bool { self.Get()._Vals.Push( arr.ParseF32()); true }
    
    #[inline( always)]
    fn	EndVt( &self) -> bool {
        let  	cnt = self.Get()._Vals.Size();
        if cnt >= U32( 2) {
            let  	w = if cnt >= U32( 3) { *self.Get()._Vals.Arr().At( 2) } else { 0.0 };
            self.Get()._VtStash.Push( WPt2f::WithW( *self.Get()._Vals.Arr().At( 0), *self.Get()._Vals.Arr().At( 1), w));
        } else if cnt == U32( 1) {
            self.Get()._VtStash.Push( WPt2f::New( *self.Get()._Vals.Arr().At( 0), 0.0));
        }
        self.Get()._Vals.Clear();
        true
    }
    
    #[inline( always)]
    fn	EndVn( &self) -> bool {
        if self.Get()._Vals.Size() >= U32( 3) {
            self.Get()._VnStash.Push( Dir3f::New( *self.Get()._Vals.Arr().At( 0), *self.Get()._Vals.Arr().At( 1), *self.Get()._Vals.Arr().At( 2)));
        }
        self.Get()._Vals.Clear();
        true
    }
    
    #[inline( always)]
    fn	EndV( &self) -> bool {
        let  	cnt = self.Get()._Vals.Size();
        if cnt >= U32( 3) {
            let  	w = if cnt >= U32( 4) { *self.Get()._Vals.Arr().At( 3) } else { 1.0 };
            self.Get()._VStash.Push( WPt3f::WithW( *self.Get()._Vals.Arr().At( 0), *self.Get()._Vals.Arr().At( 1), *self.Get()._Vals.Arr().At( 2), w));
        }
        self.Get()._Vals.Clear();
        true
    }
    
    #[inline( always)]
    fn	ParseFaceV( &self, arr: Arr< '_, U8>) -> bool {
        let  	v = arr.ParseI32();
        let  	numV = self.Get()._VStash.Size().0 as i32;
        self.Get()._CurFaceVert._VertexIdx = if v < 0 { numV + v + 1 } else { v };
        true
    }
    
    #[inline( always)]
    fn	ParseFaceVt( &self, arr: Arr< '_, U8>) -> bool {
        let  	v = arr.ParseI32();
        let  	numT = self.Get()._VtStash.Size().0 as i32;
        self.Get()._CurFaceVert._TexCoordIdx = Some( if v < 0 { numT + v + 1 } else { v });
        true
    }
    
    #[inline( always)]
    fn	ParseFaceVn( &self, arr: Arr< '_, U8>) -> bool {
        let  	v = arr.ParseI32();
        let  	numN = self.Get()._VnStash.Size().0 as i32;
        self.Get()._CurFaceVert._NormalIdx = Some( if v < 0 { numN + v + 1 } else { v });
        true
    }
    
    #[inline( always)]
    fn	EndFaceVertex( &self) -> bool {
        self.Get()._FaceVerts.Push( self.Get()._CurFaceVert);
        self.Get()._CurFaceVert = FaceVertex { _VertexIdx: 0, _TexCoordIdx: None, _NormalIdx: None };
        true
    }
    
    #[inline( always)]
    fn	EndFace( &self) -> bool {
        if self.Get()._FaceVerts.Size() > U32( 0) {
            self.Get()._FStash.Push( Face { _Vertices: self.Get()._FaceVerts.ToBuff() });
            self.Get()._FaceVerts.Clear();
        }
        true
    }
    
    #[inline( always)]
    fn	PushObj( &self, arr: Arr< '_, U8>) -> bool { self.Get()._ObjStash.Push( arr.AsStr().to_string()); true }
    
    #[inline( always)]
    fn	PushGrp( &self, arr: Arr< '_, U8>) -> bool { self.Get()._GrpStash.Push( arr.AsStr().to_string()); true }
}

//---------------------------------------------------------------------------------------------------------------------------------

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

        let  	streamSz = parser.InStream().Size();
        let  	mut ctx = WaveObjParserCtx::New( streamSz);
        let  	ctxMM = WaveObjParserCtxMM( &mut ctx as *mut _);

        let  	nonWs = *Charset::NonSpace();
        let  	nonEnd = Charset::EndLine().Negative();

        let  	objGrammar = ShardTree!(
            *(
                *[ " \t" ]
                < (
                    ( "mtllib" < +[ " \t" ] < (+nonWs)[ |arr| ctxMM.PushMtlLib( arr) ] )
                    | ( "usemtl" < +[ " \t" ] < (+nonWs)[ |arr| ctxMM.PushUseMtl( arr) ] )
                    | ( ( "vt" < +[ " \t" ] < Real[ |arr| ctxMM.PushVal( arr) ] < ?( +[ " \t" ] < Real[ |arr| ctxMM.PushVal( arr) ] < ?( +[ " \t" ] < Real[ |arr| ctxMM.PushVal( arr) ] ) ) )[ |_arr| ctxMM.EndVt() ] )
                    | ( ( "vn" < +[ " \t" ] < Real[ |arr| ctxMM.PushVal( arr) ] < +[ " \t" ] < Real[ |arr| ctxMM.PushVal( arr) ] < +[ " \t" ] < Real[ |arr| ctxMM.PushVal( arr) ] )[ |_arr| ctxMM.EndVn() ] )
                    | ( ( "v" < +[ " \t" ] < Real[ |arr| ctxMM.PushVal( arr) ] < +[ " \t" ] < Real[ |arr| ctxMM.PushVal( arr) ] < +[ " \t" ] < Real[ |arr| ctxMM.PushVal( arr) ] < ?( +[ " \t" ] < Real[ |arr| ctxMM.PushVal( arr) ] ) )[ |_arr| ctxMM.EndV() ] )
                    | ( ( "f" < +( +[ " \t" ] < (
                        Int[ |arr| ctxMM.ParseFaceV( arr) ]
                        < ?( "/"
                             < ?Int[ |arr| ctxMM.ParseFaceVt( arr) ]
                             < ?( "/"
                                  < Int[ |arr| ctxMM.ParseFaceVn( arr) ]
                             )
                        )
                    )[ |_arr| ctxMM.EndFaceVertex() ] ) )[ |_arr| ctxMM.EndFace() ] )
                    | ( "o" < +[ " \t" ] < (+nonWs)[ |arr| ctxMM.PushObj( arr) ] )
                    | ( "g" < +[ " \t" ] < (+nonWs)[ |arr| ctxMM.PushGrp( arr) ] )
                    | *nonEnd
                )
                < *[ " \t" ]
                < ?( ( ?'\r' < '\n' ) | '\r' )
            )
        );

        if parser.ParseGrammar( &objGrammar, parser.CurrMark() ).is_none() {
            return false;
        }

        model._Vertices = ctx._VStash.IntoBuff();
        model._TexCoords = ctx._VtStash.IntoBuff();
        model._Normals = ctx._VnStash.IntoBuff();
        model._Faces = ctx._FStash.IntoBuff();
        model._Objects = ctx._ObjStash.IntoBuff();
        model._Groups = ctx._GrpStash.IntoBuff();
        model._MtlLibs = ctx._MtlStash.IntoBuff();
        model._UseMtls = ctx._UseMtlStash.IntoBuff();

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




