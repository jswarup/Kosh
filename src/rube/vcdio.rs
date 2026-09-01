//-- vcdio.rs -----------------------------------------------------------------------------------------------------------------------
#![allow( dead_code, unused_variables)]

use	crate::{
    flux::instream::FixedStream,
    shard::{ Charset, IGrammar, Parser },
    silo::{ Arr, Buff, Stash, U32, U8 },
    ShardTree,
};

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone, Debug)]
pub struct VcdVar
{
    pub _Type: String,
    pub _Bits: U32,
    pub _Id: String,
    pub _Name: String,
}

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone, Debug)]
pub struct VcdScope
{
    pub _Type: String,
    pub _Name: String,
    pub _Vars: Buff< VcdVar>,
    pub _Scopes: Buff< VcdScope>,
}

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone, Debug)]
pub struct VcdValue
{
    pub _Id: String,
    pub _ValStr: String,
}

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone, Debug)]
pub struct VcdTimeStep
{
    pub _Time: u64,
    pub _Values: Buff< VcdValue>,
}

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone, Debug)]
pub struct VcdModel
{
    pub _Version: String,
    pub _Date: String,
    pub _Timescale: String,
    pub _Scopes: Buff< VcdScope>,
    pub _TimeSteps: Buff< VcdTimeStep>,
}

impl Default for VcdModel
{
    fn	default() -> Self
    {
        Self::New()
    }
}

impl VcdModel
{
    pub fn	New() -> Self
    {
        Self {
            _Version: String::new(),
            _Date: String::new(),
            _Timescale: String::new(),
            _Scopes: Buff::New(),
            _TimeSteps: Buff::New(),
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub(crate) struct VcdParserCtx
{
    _Version: String,
    _Date: String,
    _Timescale: String,
    
    _ScopeStash: Stash< VcdScope>,
    _VarStash: Stash< VcdVar>,
    
    _TimeSteps: Stash< VcdTimeStep>,
    _CurrentTime: u64,
    _CurrentValues: Stash< VcdValue>,

    _TempStash: Stash< String>,
}

impl VcdParserCtx
{
    fn	New() -> Self
    {
        Self {
            _Version: String::new(),
            _Date: String::new(),
            _Timescale: String::new(),
            _ScopeStash: Stash::New(),
            _VarStash: Stash::New(),
            _TimeSteps: Stash::New(),
            _CurrentTime: 0,
            _CurrentValues: Stash::New(),
            _TempStash: Stash::New(),
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone, Copy)]
struct VcdParserCtxMM( *mut VcdParserCtx);

impl VcdParserCtxMM
{
    #[inline( always)]
    #[allow( clippy::mut_from_ref)]
    fn	Get( &self) -> &mut VcdParserCtx
    {
        unsafe { &mut *self.0 }
    }

    #[inline( always)]
    fn	PushTemp( &self, arr: Arr< '_, U8>) -> bool
    {
        self.Get()._TempStash.Push( arr.AsStr().to_string());
        true
    }

    #[inline( always)]
    fn	SetVersion( &self) -> bool
    {
        let  	ctx = self.Get();
        if ctx._TempStash.Size() > U32( 0) {
            ctx._Version = ctx._TempStash[U32( 0)].clone();
            ctx._TempStash.Clear();
        }
        true
    }

    #[inline( always)]
    fn	SetTimescale( &self) -> bool
    {
        let  	ctx = self.Get();
        if ctx._TempStash.Size() > U32( 0) {
            ctx._Timescale = ctx._TempStash.Slice().join( " ");
            ctx._TempStash.Clear();
        }
        true
    }

    #[inline( always)]
    fn	SetDate( &self) -> bool
    {
        let  	ctx = self.Get();
        if ctx._TempStash.Size() > U32( 0) {
            ctx._Date = ctx._TempStash.Slice().join( " ");
            ctx._TempStash.Clear();
        }
        true
    }

    #[inline( always)]
    fn	PushScope( &self) -> bool
    {
        let  	ctx = self.Get();
        if ctx._TempStash.Size() >= U32( 2) {
            let  	s = VcdScope {
                _Type: ctx._TempStash[U32( 0)].clone(),
                _Name: ctx._TempStash[U32( 1)].clone(),
                _Vars: Buff::New(),
                _Scopes: Buff::New(),
            };
            ctx._ScopeStash.Push( s);
        }
        ctx._TempStash.Clear();
        true
    }

    #[inline( always)]
    fn	PushVar( &self) -> bool
    {
        let  	ctx = self.Get();
        if ctx._TempStash.Size() >= U32( 4) {
            let  	v = VcdVar {
                _Type: ctx._TempStash[U32( 0)].clone(),
                _Bits: U32( ctx._TempStash[U32( 1)].parse().unwrap_or( 1)),
                _Id: ctx._TempStash[U32( 2)].clone(),
                _Name: ctx._TempStash[U32( 3)].clone(),
            };
            ctx._VarStash.Push( v);
        }
        ctx._TempStash.Clear();
        true
    }

    #[inline( always)]
    fn	PopScope( &self) -> bool
    {
        let  	ctx = self.Get();
        // In a real hierarchical parser we'd build the tree.
        // For provisions, we just collect scopes and vars linearly.
        true
    }

    #[inline( always)]
    fn	EndDefinitions( &self) -> bool
    {
        true
    }

    #[inline( always)]
    fn	AddTime( &self, arr: Arr< '_, U8>) -> bool
    {
        let  	ctx = self.Get();
        // Push previous time step if it has values
        if ctx._CurrentValues.Size() > U32( 0) || ctx._CurrentTime == 0 {
            let  	ts = VcdTimeStep {
                _Time: ctx._CurrentTime,
                _Values: ctx._CurrentValues.ToBuff(),
            };
            ctx._TimeSteps.Push( ts);
            ctx._CurrentValues.Clear();
        }
        ctx._CurrentTime = arr.AsStr().parse().unwrap_or( 0);
        true
    }

    #[inline( always)]
    fn	AddScalarVal( &self, arr: Arr< '_, U8>) -> bool
    {
        let  	ctx = self.Get();
        let  	s = arr.AsStr();
        if s.len() >= 2 {
            let  	valStr = s[0..1].to_string();
            let  	idStr = s[1..].to_string();
            ctx._CurrentValues.Push( VcdValue { _Id: idStr, _ValStr: valStr });
        }
        true
    }

    #[inline( always)]
    fn	PushVectorVal( &self, arr: Arr< '_, U8>) -> bool
    {
        self.Get()._TempStash.Push( arr.AsStr().to_string());
        true
    }

    #[inline( always)]
    fn	PushVectorId( &self, arr: Arr< '_, U8>) -> bool
    {
        let  	ctx = self.Get();
        if ctx._TempStash.Size() > U32( 0) {
            let  	valStr = ctx._TempStash[U32( 0)].clone();
            let  	idStr = arr.AsStr().to_string();
            ctx._CurrentValues.Push( VcdValue { _Id: idStr, _ValStr: valStr });
            ctx._TempStash.Clear();
        }
        true
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub struct VcdShard< 'a>
{
    pub _Model: &'a mut VcdModel,
}

impl< 'a> IGrammar for VcdShard< 'a>
{
    fn	Match( &self, parser: &mut Parser) -> bool
    {
        let  	modelPtr: *mut VcdModel = self._Model as *const VcdModel as *mut VcdModel;
        let  	model = unsafe { &mut *modelPtr };

        let  	mut ctx = VcdParserCtx::New();
        let  	ctxMM = VcdParserCtxMM( &mut ctx as *mut _);

        let  	vcdGrammar = ShardTree!(
            *( *Charset::All() )
        );

        if parser.ParseGrammar( &vcdGrammar, parser.CurrMark()).is_none() {
            return false;
        }

        // We successfully consumed the file, but we didn't populate the model.
        // Full VCD grammar implementation is deferred.

        // Flush last time step
        if ctx._CurrentValues.Size() > U32( 0) {
            let  	ts = VcdTimeStep {
                _Time: ctx._CurrentTime,
                _Values: ctx._CurrentValues.ToBuff(),
            };
            ctx._TimeSteps.Push( ts);
        }

        model._Version = ctx._Version;
        model._Date = ctx._Date;
        model._Timescale = ctx._Timescale;
        model._Scopes = ctx._ScopeStash.IntoBuff();
        model._TimeSteps = ctx._TimeSteps.IntoBuff();

        true
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub fn	ParseVcd( input: &str) -> Result< VcdModel, String>
{
    let  	mut stream = FixedStream::from( input);
    let  	mut model = VcdModel::New();
    let  	mut parser = Parser::New( &mut stream);
    let  	shard = VcdShard { _Model: &mut model };
    let  	res = parser.ParseGrammar( &shard, U32( 0));
    if res.is_some() {
        Ok( model)
    } else {
        Err( "Failed to parse VCD stream".to_string())
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
