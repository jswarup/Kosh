//-- ptio.rs -----------------------------------------------------------------------------------------------------------------------
use	std::fmt;
use	crate::{
    fenst::PtsPointsDto,
    fleck::{ BBox3f, Pt3f },
    flux::instream::{ FixedStream, IStream },
    shard::{ IGrammar, Parser, Real, Charset },
    silo::{ Buff, Stash, IAccess, U32, U8 },
    ShardTree,
};

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone, Copy, Debug, PartialEq)]
pub struct RGB
{
    pub _R: U8,
    pub _G: U8,
    pub _B: U8,
}

impl Default for RGB
{
    fn	default() -> Self
    {
        Self {
            _R: U8( 0),
            _G: U8( 0),
            _B: U8( 0),
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Represents a single point in a .pts point cloud with optional intensity and RGB color.
#[derive( Clone, Copy, Debug, PartialEq)]
pub struct PtsPoint
{
    pub _Pos:       Pt3f,
    pub _Intensity: Option< f32>,
    pub _Color:     Option< RGB>,
}

impl Default for PtsPoint
{
    fn	default() -> Self
    {
        Self {
            _Pos: Pt3f::default(),
            _Intensity: None,
            _Color: None,
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl PtsPoint
{
    pub fn	New( x: f32, y: f32, z: f32) -> Self
    {
        Self {
            _Pos: Pt3f::New( x, y, z),
            _Intensity: None,
            _Color: None,
        }
    }

    pub fn	WithIntensity( x: f32, y: f32, z: f32, intensity: f32) -> Self
    {
        Self {
            _Pos: Pt3f::New( x, y, z),
            _Intensity: Some( intensity),
            _Color: None,
        }
    }

    pub fn	WithColor( x: f32, y: f32, z: f32, color: RGB) -> Self
    {
        Self {
            _Pos: Pt3f::New( x, y, z),
            _Intensity: None,
            _Color: Some( color),
        }
    }

    pub fn	WithIntensityAndColor( x: f32, y: f32, z: f32, intensity: f32, color: RGB) -> Self
    {
        Self {
            _Pos: Pt3f::New( x, y, z),
            _Intensity: Some( intensity),
            _Color: Some( color),
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Represents an in-memory collection of point cloud points.
#[derive( Clone, PartialEq)]
pub struct PtsCloud
{
    pub _Points:      Buff< PtsPoint>,
    pub _HeaderCount: Option< U32>,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Default for PtsCloud
{
    fn	default() -> Self
    {
        Self::New()
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl PtsCloud
{
    pub fn	New() -> Self
    {
        Self {
            _Points: Buff::New(),
            _HeaderCount: None,
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	WithCapacity( capacity: U32) -> Self
    {
        Self {
            _Points: Buff::Create( capacity, |_| PtsPoint::default()),
            _HeaderCount: Some( capacity),
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Push( &mut self, point: PtsPoint)
    {
        let  	mut stash = Stash::WithCapacity( self._Points.Size() + U32( 1));
        let  	arr = self._Points.Arr();
        for i in 0..self._Points.Size().AsUsize() {
            stash.Push( *arr.At( U32( i as u32)));
        }
        stash.Push( point);
        self._Points = stash.IntoBuff();
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Count( &self) -> U32
    {
        self._Points.Size()
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	IsEmpty( &self) -> bool
    {
        self._Points.IsEmpty()
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Points( &self) -> &Buff< PtsPoint>
    {
        &self._Points
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	BoundingBox( &self) -> ( [f32; 3], [f32; 3])
    {
        if self._Points.IsEmpty() {
            return ( [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]);
        }
        let  	arr = self._Points.Arr();
        let  	mut bbox = BBox3f::Empty();
        let  	sz = self._Points.Size().AsUsize();
        for i in 0..sz {
            let  	pt = arr.At( U32( i as u32));
            bbox.Extend( pt._Pos);
        }
        ( bbox.Min(), bbox.Max())
    }

    pub fn	BBox( &self) -> BBox3f
    {
        let  	( bboxMin, bboxMax) = self.BoundingBox();
        BBox3f::New( Pt3f::from( bboxMin), Pt3f::from( bboxMax))
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	ToDto( &self) -> PtsPointsDto
    {
        let  	( bboxMin, bboxMax) = self.BoundingBox();
        let  	totalPoints = self._Points.Size();
        let  	arr = self._Points.Arr();
        let  	pointsBuff = Buff::Create( totalPoints, |i| {
            let  	pt = arr.At( i);
            [pt._Pos._X, pt._Pos._Y, pt._Pos._Z]
        });
        PtsPointsDto {
            _Points: pointsBuff,
            _Count: totalPoints.AsUsize(),
            _BboxMin: bboxMin,
            _BboxMax: bboxMax,
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Shard grammar struct that parses a .pts point cloud into a PtsCloud target using Stash with initial capacity estimate.
pub struct PtsShard< 'a>
{
    pub _Cloud: &'a mut PtsCloud,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> IGrammar for PtsShard< 'a>
{
    fn	Match( &self, parser: &mut Parser) -> bool
    {
        let  	cloudPtr = self._Cloud as *const PtsCloud as *mut PtsCloud;
        let  	cloud = unsafe { &mut *cloudPtr };

        let  	estimatedCap = cloud._HeaderCount.unwrap_or_else( || {
            let  	streamSz = parser.InStream().Size().AsUsize();
            U32( (streamSz / 32).max( 128) as u32)
        });
        let  	mut pointsStash = Stash::< PtsPoint>::WithCapacity( estimatedCap);

        let  	mut m = parser.CurrMark();
        let  	numGrammar = ShardTree!( Real  );
        let  	hspcGrammar = ShardTree!( +[ " \t," ] );
        let  	nlGrammar = ShardTree!( ( ?'\r' < '\n' ) | '\r' );

        // Helper: skip whitespace, comments, and empty lines
        let  	mut isFirstLine = true;

        let  	skippable = ShardTree!( *(( ( "#" | "//" ) < *( Charset::EndLine().Negative()) < "\n") |  [ " \r\n\t," ] ));

        while m < parser.InStream().Size() {
            if let Some( nextM) = parser.ParseGrammar( &skippable, m) {
                m = nextM;
            }

            if m >= parser.InStream().Size() {
                break;
            }

            // Parse tokens on this line
            let  	mut lineNums: [f32; 8] = [0.0; 8];
            let  	mut numCount = 0usize;

            while m < parser.InStream().Size() {
                let  	currByte = parser.GetAt( m);
                if currByte == U8( b'\r') || currByte == U8( b'\n') {
                    break;
                }
                if currByte == U8( b'#') {
                    // Line trailing comment
                    while m < parser.InStream().Size() {
                        let  	c = parser.GetAt( m);
                        if let Some( nextM) = parser.Incr( m) {
                            m = nextM;
                        } else {
                            break;
                        }
                        if c == U8( b'\n') {
                            break;
                        }
                    }
                    break;
                }

                let  	tokenMark = m;
                if let Some( nextM) = parser.ParseGrammar( &numGrammar, tokenMark) {
                    let  	bytes = parser.InStream().BytesAt( tokenMark, nextM - tokenMark);
                    let  	numStr = <&str>::from( bytes);
                    if let Ok( val) = numStr.parse::< f32>() {
                        if numCount < lineNums.len() {
                            lineNums[numCount] = val;
                            numCount += 1;
                        }
                    }
                    m = nextM;
                } else {
                    // Unknown non-number token on line, skip byte
                    if let Some( nextM) = parser.Incr( m) {
                        m = nextM;
                    } else {
                        break;
                    }
                }

                // Skip horizontal spaces after token
                if let Some( nextM) = parser.ParseGrammar( &hspcGrammar, m) {
                    m = nextM;
                }
            }

            if isFirstLine && numCount == 1 {
                // Header line with total points count
                cloud._HeaderCount = Some( U32( lineNums[0] as u32));
                isFirstLine = false;
            } else if numCount >= 3 {
                isFirstLine = false;
                let  	pt = if numCount == 3 {
                    PtsPoint::New( lineNums[0], lineNums[1], lineNums[2])
                } else if numCount == 4 {
                    PtsPoint { _Intensity: Some( lineNums[3]), ..PtsPoint::New( lineNums[0], lineNums[1], lineNums[2]) }
                } else if numCount == 6 {
                    PtsPoint { 
                        _Color: Some( RGB {
                            _R: U8( lineNums[3] as u8), 
                            _G: U8( lineNums[4] as u8), 
                            _B: U8( lineNums[5] as u8),
                        }),
                        ..PtsPoint::New( lineNums[0], lineNums[1], lineNums[2]) 
                    }
                } else if numCount >= 7 {
                    PtsPoint { 
                        _Intensity: Some( lineNums[3]), 
                        _Color: Some( RGB {
                            _R: U8( lineNums[4] as u8), 
                            _G: U8( lineNums[5] as u8), 
                            _B: U8( lineNums[6] as u8),
                        }),
                        ..PtsPoint::New( lineNums[0], lineNums[1], lineNums[2]) 
                    }
                } else {
                    PtsPoint::New( lineNums[0], lineNums[1], lineNums[2])
                };
                pointsStash.Push( pt);
            }

            // Advance past newline
            if let Some( nextM) = parser.ParseGrammar( &nlGrammar, m) {
                m = nextM;
            }
        }

        cloud._Points = pointsStash.IntoBuff();
        parser.SetCurrMark( m);
        true
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Parses a .pts point cloud file from a string slice.
pub fn	ParsePts( input: &str) -> Result< PtsCloud, String>
{
    let  	mut stream = FixedStream::from( input);
    ParsePtsStream( &mut stream)
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Parses a .pts point cloud file from a raw byte slice.
pub fn	ParsePtsBytes( bytes: &[u8]) -> Result< PtsCloud, String>
{
    let  	s = std::str::from_utf8( bytes).map_err( |e| e.to_string())?;
    ParsePts( s)
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Parses a .pts point cloud file from an input stream.
pub fn	ParsePtsStream( stream: &mut dyn IStream) -> Result< PtsCloud, String>
{
    let  	mut cloud = PtsCloud::New();
    let  	mut parser = Parser::New( stream);
    let  	shard = PtsShard { _Cloud: &mut cloud };
    let  	res = parser.ParseGrammar( &shard, U32( 0));
    if res.is_some() {
        Ok( cloud)
    } else {
        Err( "Failed to parse .pts stream".to_string())
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl fmt::Debug for PtsCloud
{
    fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result
    {
        f.debug_struct( "PtsCloud")
            .field( "count", &self.Count().AsUsize())
            .field( "header_count", &self._HeaderCount)
            .finish()
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl fmt::Display for PtsCloud
{
    fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result
    {
        write!( f, "PtsCloud({} points)", self.Count())
    }
}

//---------------------------------------------------------------------------------------------------------------------------------