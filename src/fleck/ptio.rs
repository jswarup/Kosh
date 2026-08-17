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
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive( Clone, Copy, Debug, PartialEq)]
pub struct RGB
{
    pub r: U8,
    pub g: U8,
    pub b: U8,
}

/// Represents a single 3D point in a .pts point cloud with optional intensity and RGB color.
#[derive( Clone, Copy, Debug, PartialEq)]
pub struct PtsPoint
{
    pub pos:        Point32,
    pub intensity:  Option< f32>,
    pub color:      Option< RGB>,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl PtsPoint
{
    pub fn	New( x: f32, y: f32, z: f32) -> Self
    {
        Self {
            pos: Point32 { x, y, z },
            intensity: None,
            color: None,
        }
    }

    pub fn	Pos( &self) -> [f32; 3]
    {
        [self.pos.x, self.pos.y, self.pos.z]
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
            _Points: Buff::NewEmpty(),
            _HeaderCount: None,
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	WithCapacity( capacity: U32) -> Self
    {
        Self {
            _Points: Buff::NewEmpty(),
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
        let  	mut minX = first.pos.x;
        let  	mut minY = first.pos.y;
        let  	mut minZ = first.pos.z;
        let  	mut maxX = first.pos.x;
        let  	mut maxY = first.pos.y;
        let  	mut maxZ = first.pos.z;
        for i in 1..self._Points.Size().AsUsize() {
            let  	pt = arr.At( U32( i as u32));
            if pt.pos.x < minX { minX = pt.pos.x; }
            if pt.pos.y < minY { minY = pt.pos.y; }
            if pt.pos.z < minZ { minZ = pt.pos.z; }
            if pt.pos.x > maxX { maxX = pt.pos.x; }
            if pt.pos.y > maxY { maxY = pt.pos.y; }
            if pt.pos.z > maxZ { maxZ = pt.pos.z; }
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
            [pt.pos.x, pt.pos.y, pt.pos.z]
        });
        PtsPointsDto {
            points: pointsBuff,
            count: totalPoints.AsUsize(),
            bbox_min: bboxMin,
            bbox_max: bboxMax,
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

            // Check if first line was a point count header (single integer count on line)
            if isFirstLine && numCount == 1 {
                cloud._HeaderCount = Some( U32( lineNums[0] as u32));
                isFirstLine = false;
            } else if numCount >= 3 {
                isFirstLine = false;
                let  	pt = if numCount == 3 {
                    PtsPoint::New( lineNums[0], lineNums[1], lineNums[2])
                } else if numCount == 4 {
                    PtsPoint { intensity: Some( lineNums[3]), ..PtsPoint::New( lineNums[0], lineNums[1], lineNums[2]) }
                } else if numCount == 6 {
                    PtsPoint { 
                        color: Some( RGB {
                            r: U8( lineNums[3] as u8), 
                            g: U8( lineNums[4] as u8), 
                            b: U8( lineNums[5] as u8),
                        }),
                        ..PtsPoint::New( lineNums[0], lineNums[1], lineNums[2]) 
                    }
                } else if numCount >= 7 {
                    PtsPoint { 
                        intensity: Some( lineNums[3]), 
                        color: Some( RGB {
                            r: U8( lineNums[4] as u8), 
                            g: U8( lineNums[5] as u8), 
                            b: U8( lineNums[6] as u8),
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
