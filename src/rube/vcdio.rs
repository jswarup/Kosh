//-- vcdio.rs -----------------------------------------------------------------------------------------------------------------------
#![allow( dead_code, unused_variables)]

use	crate::{
    flux::instream::FixedStream,
    shard::{ Charset, IGrammar, Int, Parser },
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

struct ScopeBuilder
{
    _Type:        String,
    _Name:        String,
    _Vars:        Stash< VcdVar>,
    _ChildScopes: Stash< VcdScope>,
}

pub(crate) struct VcdParserCtx
{
    _Version:       String,
    _Date:          String,
    _Timescale:     String,

    _ScopeStack:    Stash< ScopeBuilder>,
    _RootScopes:    Stash< VcdScope>,

    _TimeSteps:     Stash< VcdTimeStep>,
    _CurrentTime:   u64,
    _CurrentValues: Stash< VcdValue>,

    _TempStash:     Stash< String>,
}

impl VcdParserCtx
{
    fn	New() -> Self
    {
        Self {
            _Version:       String::new(),
            _Date:          String::new(),
            _Timescale:     String::new(),
            _ScopeStack:    Stash::New(),
            _RootScopes:    Stash::New(),
            _TimeSteps:     Stash::New(),
            _CurrentTime:   0,
            _CurrentValues: Stash::New(),
            _TempStash:     Stash::New(),
        }
    }

    fn	DrainScopeStack( &mut self)
    {
        while self._ScopeStack.Size() > U32( 0) {
            let  	popped = self._ScopeStack.Pop().unwrap();
            let  	scope = VcdScope {
                _Type:   popped._Type,
                _Name:   popped._Name,
                _Vars:   popped._Vars.IntoBuff(),
                _Scopes: popped._ChildScopes.IntoBuff(),
            };
            if self._ScopeStack.Size() > U32( 0) {
                let  	parentIdx = self._ScopeStack.Size() - U32( 1);
                self._ScopeStack[parentIdx]._ChildScopes.Push( scope);
            } else {
                self._RootScopes.Push( scope);
            }
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
            let  	builder = ScopeBuilder {
                _Type:        ctx._TempStash[U32( 0)].clone(),
                _Name:        ctx._TempStash[U32( 1)].clone(),
                _Vars:        Stash::New(),
                _ChildScopes: Stash::New(),
            };
            ctx._ScopeStack.Push( builder);
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
                _Id:   ctx._TempStash[U32( 2)].clone(),
                _Name: ctx._TempStash[U32( 3)].clone(),
            };
            if ctx._ScopeStack.Size() > U32( 0) {
                let  	topIdx = ctx._ScopeStack.Size() - U32( 1);
                ctx._ScopeStack[topIdx]._Vars.Push( v);
            }
        }
        ctx._TempStash.Clear();
        true
    }

    #[inline( always)]
    fn	PopScope( &self) -> bool
    {
        let  	ctx = self.Get();
        if ctx._ScopeStack.Size() > U32( 0) {
            let  	popped = ctx._ScopeStack.Pop().unwrap();
            let  	scope = VcdScope {
                _Type:   popped._Type,
                _Name:   popped._Name,
                _Vars:   popped._Vars.IntoBuff(),
                _Scopes: popped._ChildScopes.IntoBuff(),
            };
            if ctx._ScopeStack.Size() > U32( 0) {
                let  	parentIdx = ctx._ScopeStack.Size() - U32( 1);
                ctx._ScopeStack[parentIdx]._ChildScopes.Push( scope);
            } else {
                ctx._RootScopes.Push( scope);
            }
        }
        true
    }

    #[inline( always)]
    fn	EndDefinitions( &self) -> bool
    {
        self.Get().DrainScopeStack();
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

        let  	nonWs = *Charset::NonSpace();
        let  	notDollar = Charset::from( &b"$"[..]).Negative();
        let  	notDollarWs = notDollar & nonWs;

        let  	vcdGrammar = ShardTree!(
            *(
                *[ " \t\r\n" ]
                < (
                    ( "$version" < *( notDollar )[ |arr| ctxMM.PushTemp( arr) ] < "$end"[ |_arr| ctxMM.SetVersion() ] )
                    | ( "$timescale" < *( notDollar )[ |arr| ctxMM.PushTemp( arr) ] < "$end"[ |_arr| ctxMM.SetTimescale() ] )
                    | ( "$date" < *( notDollar )[ |arr| ctxMM.PushTemp( arr) ] < "$end"[ |_arr| ctxMM.SetDate() ] )
                    | ( "$scope" < +( +[ " \t\r\n" ] < (+notDollarWs)[ |arr| ctxMM.PushTemp( arr) ] ) < *[ " \t\r\n" ] < "$end"[ |_arr| ctxMM.PushScope() ] )
                    | ( "$var" < +( +[ " \t\r\n" ] < (+notDollarWs)[ |arr| ctxMM.PushTemp( arr) ] ) < *[ " \t\r\n" ] < "$end"[ |_arr| ctxMM.PushVar() ] )
                    | ( "$upscope" < *[ " \t\r\n" ] < "$end"[ |_arr| ctxMM.PopScope() ] )
                    | ( "$enddefinitions" < *[ " \t\r\n" ] < "$end"[ |_arr| ctxMM.EndDefinitions() ] )
                    | ( "$dumpvars" )
                    | ( "$end" )
                    | ( "#" < Int[ |arr| ctxMM.AddTime( arr) ] )
                    | ( ( "b" | "B" ) < (+[ "01xXzZ" ])[ |arr| ctxMM.PushVectorVal( arr) ] < *[ " \t\r\n" ] < (+nonWs)[ |arr| ctxMM.PushVectorId( arr) ] )
                    | ( ( [ "01xXzZ" ] < +nonWs )[ |arr| ctxMM.AddScalarVal( arr) ] )
                )
                < *[ " \t\r\n" ]
            )
        );

        if parser.ParseGrammar( &vcdGrammar, parser.CurrMark()).is_none() {
            return false;
        }

        ctx.DrainScopeStack();

        // Flush last time step
        if ctx._CurrentValues.Size() > U32( 0) {
            let  	ts = VcdTimeStep {
                _Time:   ctx._CurrentTime,
                _Values: ctx._CurrentValues.ToBuff(),
            };
            ctx._TimeSteps.Push( ts);
        }

        model._Version = ctx._Version;
        model._Date = ctx._Date;
        model._Timescale = ctx._Timescale;
        model._Scopes = ctx._RootScopes.IntoBuff();
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
