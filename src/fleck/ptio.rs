//-- ptsio.rs -----------------------------------------------------------------------------------------------------------------------

use	std::fmt;
use	crate::{
    fenst::PtsPointsDto,
    flux::instream::{ FixedStream, IStream },
    shard::{ IGrammar, Parser, Real, Charset },
    silo::{ Buff, IAccess, U32, U8 },
    ShardTree,
};

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone, Copy, Debug, PartialEq)]
pub struct Point32
{
    pub _X: f32,
    pub _Y: f32,
    pub _Z: f32,
}

#[derive( Clone, Copy, Debug, PartialEq)]
pub struct RGB
{
    pub _R: U8,
    pub _G: U8,
    pub _B: U8,
}

/// Represents a single 3D point in a .pts point cloud with optional intensity and RGB color.
#[derive( Clone, Copy, Debug, PartialEq)]
pub struct PtsPoint
{
    pub _Pos:        Point32,
    pub _Intensity:  Option< f32>,
    pub _Color:      Option< RGB>,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl PtsPoint
{
    pub fn	New( x: f32, y: f32, z: f32) -> Self
    {
        Self {
            _Pos: Point32 { _X: x, _Y: y, _Z: z },
            _Intensity: None,
            _Color: None,
        }
    }

    pub fn	Pos( &self) -> [f32; 3]
    {
        [self._Pos._X, self._Pos._Y, self._Pos._Z]
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Represents a parsed .pts point cloud dataset.
#[derive( Clone)]
pub struct PtsCloud
{
    pub _Points:        Buff< PtsPoint>,
    pub _HeaderCount:   Option< U32>,
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
            _Points: Buff::New(),
            _HeaderCount: Some( capacity),
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Push( &mut self, point: PtsPoint)
    {
        self._Points.Push( point);
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
        let  	first = arr.At( U32( 0));
        let  	mut minX = first._Pos._X;
        let  	mut minY = first._Pos._Y;
        let  	mut minZ = first._Pos._Z;
        let  	mut maxX = first._Pos._X;
        let  	mut maxY = first._Pos._Y;
        let  	mut maxZ = first._Pos._Z;
        for i in 1..self._Points.Size().AsUsize() {
            let  	pt = arr.At( U32( i as u32));
            if pt._Pos._X < minX { minX = pt._Pos._X; }
            if pt._Pos._Y < minY { minY = pt._Pos._Y; }
            if pt._Pos._Z < minZ { minZ = pt._Pos._Z; }
            if pt._Pos._X > maxX { maxX = pt._Pos._X; }
            if pt._Pos._Y > maxY { maxY = pt._Pos._Y; }
            if pt._Pos._Z > maxZ { maxZ = pt._Pos._Z; }
        }
        ( [minX, minY, minZ], [maxX, maxY, maxZ])
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

/// Shard grammar struct that parses a .pts point cloud into a `PtsCloud` target.
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
                cloud.Push( pt);
            }

            // Advance past newline
            if let Some( nextM) = parser.ParseGrammar( &nlGrammar, m) {
                m = nextM;
            }
        }

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
