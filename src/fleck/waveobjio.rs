//-- waveobjio.rs ------------------------------------------------------------------------------------------------------------------
use	std::fmt;
use	crate::{
    fenst::{ PtsPointsDto, WaveObjMeshDto },
    fleck::{ BBox3f, Dir3f, Pt3f, WPt2f, WPt3f },
    flux::instream::{ FixedStream, IStream },
    shard::{ IGrammar, Parser, Charset, Real, Int },
    silo::{ Buff, Stash, IAccess, U32, U8, cast::IConstPtrMutRefExt },
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
        let  	mut bbox = BBox3f::Empty();
        for i in 0..self._Vertices.Size().AsUsize() {
            let  	v = arr.At( U32( i as u32));
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

/// Parses up to `maxCount` floats from the current line, storing into `nums`.
/// Returns the number of floats successfully parsed. Advances `p` past consumed tokens.
fn	ParseFloatsOnLine( p: &mut Parser, nums: &mut [f32], maxCount: usize) -> usize
{
    let  	numGrammar = ShardTree!( Real );
    let  	hspc = ShardTree!( +[" \t"] );
    let  	mut numCount = 0usize;
    let  	mut mCur = p.CurrMark();

    while numCount < maxCount && mCur < p.InStream().Size() {
        let  	b = p.GetAt( mCur);
        if b == U8( b'\r') || b == U8( b'\n') { break; }

        let  	tokenMark = mCur;
        if let Some( nextM) = p.ParseGrammar( &numGrammar, tokenMark) {
            let  	bytes = p.InStream().BytesAt( tokenMark, nextM - tokenMark);
            if let Ok( val) = <&str>::from( bytes).parse::< f32>() {
                nums[numCount] = val;
                numCount += 1;
            }
            mCur = nextM;
        } else if let Some( nextM) = p.Incr( mCur) {
            mCur = nextM;
        } else {
            break;
        }

        if let Some( nextM) = p.ParseGrammar( &hspc, mCur) {
            mCur = nextM;
        }
    }

    p.SetCurrMark( mCur);
    numCount
}

//---------------------------------------------------------------------------------------------------------------------------------

fn	ParseVertexLine( p: &mut Parser, vertPtr: *const Stash< WPt3f>) -> bool
{
    let  	mStart = p.CurrMark();
    let  	vTag = ShardTree!( "v" < +[" \t"] );
    let Some( mAfterTag) = p.ParseGrammar( &vTag, mStart) else {
        return false;
    };
    p.SetCurrMark( mAfterTag);

    let  	mut nums: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
    let  	numCount = ParseFloatsOnLine( p, &mut nums, 4);
    if numCount >= 3 {
        vertPtr.MutRef().Push( WPt3f::WithW( nums[0], nums[1], nums[2], nums[3]));
        true
    } else {
        false
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

fn	ParseTexCoordLine( p: &mut Parser, texPtr: *const Stash< WPt2f>) -> bool
{
    let  	mStart = p.CurrMark();
    let  	vtTag = ShardTree!( "vt" < +[" \t"] );
    let Some( mAfterTag) = p.ParseGrammar( &vtTag, mStart) else {
        return false;
    };
    p.SetCurrMark( mAfterTag);

    let  	mut nums: [f32; 3] = [0.0, 0.0, 0.0];
    let  	numCount = ParseFloatsOnLine( p, &mut nums, 3);
    if numCount >= 2 {
        texPtr.MutRef().Push( WPt2f::WithW( nums[0], nums[1], nums[2]));
        true
    } else if numCount == 1 {
        texPtr.MutRef().Push( WPt2f::New( nums[0], 0.0));
        true
    } else {
        false
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

fn	ParseNormalLine( p: &mut Parser, normPtr: *const Stash< Dir3f>) -> bool
{
    let  	mStart = p.CurrMark();
    let  	vnTag = ShardTree!( "vn" < +[" \t"] );
    let Some( mAfterTag) = p.ParseGrammar( &vnTag, mStart) else {
        return false;
    };
    p.SetCurrMark( mAfterTag);

    let  	mut nums: [f32; 3] = [0.0, 0.0, 0.0];
    let  	numCount = ParseFloatsOnLine( p, &mut nums, 3);
    if numCount >= 3 {
        normPtr.MutRef().Push( Dir3f::New( nums[0], nums[1], nums[2]));
        true
    } else {
        false
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

fn	ParseFaceLine( p: &mut Parser, vertPtr: *const Stash< WPt3f>, texPtr: *const Stash< WPt2f>, normPtr: *const Stash< Dir3f>, facePtr: *const Stash< Face>) -> bool
{
    let  	mStart = p.CurrMark();
    let  	fTag = ShardTree!( "f" < +[" \t"] );
    let Some( mAfterTag) = p.ParseGrammar( &fTag, mStart) else {
        return false;
    };

    let  	intGrammar = ShardTree!( Int );
    let  	hspc = ShardTree!( +[" \t"] );
    let  	numV = vertPtr.MutRef().Size().AsUsize();
    let  	numT = texPtr.MutRef().Size().AsUsize();
    let  	numN = normPtr.MutRef().Size().AsUsize();

    let  	mut faceVerts = Stash::< FaceVertex>::WithCapacity( U32( 4));
    let  	mut mCur = mAfterTag;

    while mCur < p.InStream().Size() {
        let  	b = p.GetAt( mCur);
        if b == U8( b'\r') || b == U8( b'\n') { break; }

        let  	tokenMark = mCur;
        if let Some( mAfterV) = p.ParseGrammar( &intGrammar, tokenMark) {
            let  	vBytes = p.InStream().BytesAt( tokenMark, mAfterV - tokenMark);
            let  	vRaw: i32 = <&str>::from( vBytes).parse().unwrap_or( 0);
            let  	vIdx = if vRaw < 0 { (numV as i32) + vRaw + 1 } else { vRaw };

            let  	mut vtIdx: Option< i32> = None;
            let  	mut vnIdx: Option< i32> = None;
            let  	mut mFV = mAfterV;

            let  	slashGrammar = ShardTree!( '/' );
            if let Some( mAfterSlash1) = p.ParseGrammar( &slashGrammar, mFV) {
                mFV = mAfterSlash1;
                if let Some( mAfterVt) = p.ParseGrammar( &intGrammar, mFV) {
                    let  	vtBytes = p.InStream().BytesAt( mFV, mAfterVt - mFV);
                    let  	vtRaw: i32 = <&str>::from( vtBytes).parse().unwrap_or( 0);
                    vtIdx = Some( if vtRaw < 0 { (numT as i32) + vtRaw + 1 } else { vtRaw });
                    mFV = mAfterVt;
                }
                if let Some( mAfterSlash2) = p.ParseGrammar( &slashGrammar, mFV) {
                    mFV = mAfterSlash2;
                    if let Some( mAfterVn) = p.ParseGrammar( &intGrammar, mFV) {
                        let  	vnBytes = p.InStream().BytesAt( mFV, mAfterVn - mFV);
                        let  	vnRaw: i32 = <&str>::from( vnBytes).parse().unwrap_or( 0);
                        vnIdx = Some( if vnRaw < 0 { (numN as i32) + vnRaw + 1 } else { vnRaw });
                        mFV = mAfterVn;
                    }
                }
            }

            faceVerts.Push( FaceVertex {
                _VertexIdx:   vIdx,
                _TexCoordIdx: vtIdx,
                _NormalIdx:   vnIdx,
            });
            mCur = mFV;
        } else if let Some( nextM) = p.Incr( mCur) {
            mCur = nextM;
        } else {
            break;
        }

        if let Some( nextM) = p.ParseGrammar( &hspc, mCur) {
            mCur = nextM;
        }
    }

    if faceVerts.Size() > U32( 0) {
        facePtr.MutRef().PushVal( Face { _Vertices: faceVerts.IntoBuff() });
        p.SetCurrMark( mCur);
        true
    } else {
        false
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

fn	ParseMetaLine( p: &mut Parser, objPtr: *const Stash< String>, grpPtr: *const Stash< String>, mtlPtr: *const Stash< String>, usePtr: *const Stash< String>) -> bool
{
    let  	mStart = p.CurrMark();
    let  	tags: [(&str, u8); 4] = [
        ( "mtllib", 0), ( "usemtl", 1), ( "o", 2), ( "g", 3),
    ];

    let  	mut tagId = 0u8;
    let  	mut mAfterTag = mStart;
    let  	mut matched = false;
    let  	mut tIdx = 0usize;
    while tIdx < tags.len() {
        let  	( tag, id) = tags[tIdx];
        let  	tagWithSpc = |tp: &mut Parser| -> bool {
            let  	tM = tp.CurrMark();
            if let Some( mTag) = tp.ParseGrammar( &tag, tM) {
                let  	spcGrammar = ShardTree!( +[" \t"] );
                if let Some( mSpc) = tp.ParseGrammar( &spcGrammar, mTag) {
                    tp.SetCurrMark( mSpc);
                    return true;
                }
            }
            false
        };
        if let Some( mTag) = p.ParseGrammar( &tagWithSpc, mStart) {
            tagId = id;
            mAfterTag = mTag;
            matched = true;
            break;
        }
        tIdx += 1;
    }

    if !matched {
        return false;
    }

    let  	nameGrammar = ShardTree!( +( Charset::EndLine().Negative()) );
    let  	mut mCur = mAfterTag;
    if let Some( mAfterName) = p.ParseGrammar( &nameGrammar, mCur) {
        let  	nameBytes = p.InStream().BytesAt( mCur, mAfterName - mCur);
        let  	name = <&str>::from( nameBytes).trim().to_string();
        match tagId {
            0 => mtlPtr.MutRef().PushVal( name),
            1 => usePtr.MutRef().PushVal( name),
            2 => objPtr.MutRef().PushVal( name),
            3 => grpPtr.MutRef().PushVal( name),
            _ => {}
        }
        mCur = mAfterName;
    }
    p.SetCurrMark( mCur);
    true
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
        let  	modelPtr = &self._Model as *const &mut WaveObjModel;
        let  	model = modelPtr.MutRef();

        let  	streamSz = parser.InStream().Size().AsUsize();
        let  	estVerts = (streamSz / 40).max( 128) as u32;
        let  	estFaces = (streamSz / 50).max( 128) as u32;

        let  	verticesStash = Stash::< WPt3f>::WithCapacity( U32( estVerts));
        let  	texCoordsStash = Stash::< WPt2f>::WithCapacity( U32( (estVerts / 2).max( 64)));
        let  	normalsStash = Stash::< Dir3f>::WithCapacity( U32( (estVerts / 2).max( 64)));
        let  	facesStash = Stash::< Face>::WithCapacity( U32( estFaces));
        let  	objectsStash = Stash::< String>::New();
        let  	groupsStash = Stash::< String>::New();
        let  	mtlLibsStash = Stash::< String>::New();
        let  	useMtlsStash = Stash::< String>::New();

        // Raw pointers for interior mutability in Fn closures (IConstPtrMutRefExt pattern)
        let  	vertPtr = &verticesStash as *const Stash< WPt3f>;
        let  	texPtr = &texCoordsStash as *const Stash< WPt2f>;
        let  	normPtr = &normalsStash as *const Stash< Dir3f>;
        let  	facePtr = &facesStash as *const Stash< Face>;
        let  	objPtr = &objectsStash as *const Stash< String>;
        let  	grpPtr = &groupsStash as *const Stash< String>;
        let  	mtlPtr = &mtlLibsStash as *const Stash< String>;
        let  	usePtr = &useMtlsStash as *const Stash< String>;

        let  	mut m = parser.CurrMark();
        let  	nlGrammar = ShardTree!( ( ?'\r' < '\n' ) | '\r' );
        let  	commentLine = ShardTree!( '#' < *( Charset::EndLine().Negative()) < nlGrammar );

        //--- Top-level: skip whitespace/comments, dispatch directives ---

        let  	skippable = ShardTree!( *( commentLine | [" \r\n\t"] ) );

        let  	directive = ShardTree!(
            (|p: &mut Parser| ParseTexCoordLine( p, texPtr))
            | (|p: &mut Parser| ParseNormalLine( p, normPtr))
            | (|p: &mut Parser| ParseVertexLine( p, vertPtr))
            | (|p: &mut Parser| ParseFaceLine( p, vertPtr, texPtr, normPtr, facePtr))
            | (|p: &mut Parser| ParseMetaLine( p, objPtr, grpPtr, mtlPtr, usePtr))
        );

        while m < parser.InStream().Size() {
            if let Some( nextM) = parser.ParseGrammar( &skippable, m) {
                m = nextM;
            }
            if m >= parser.InStream().Size() { break; }

            if let Some( nextM) = parser.ParseGrammar( &directive, m) {
                m = nextM;
            } else {
                // Unknown line — skip to end of line
                while m < parser.InStream().Size() {
                    let  	b = parser.GetAt( m);
                    if b == U8( b'\r') || b == U8( b'\n') { break; }
                    if let Some( nextM) = parser.Incr( m) {
                        m = nextM;
                    } else {
                        break;
                    }
                }
                if let Some( nextM) = parser.ParseGrammar( &nlGrammar, m) {
                    m = nextM;
                }
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
