//-- fluxbasics.rs -----------------------------------------------------------------------------------------------------------------------

use crate::flux::{ IFluxExportSource, fluxexport::FieldExp };
use crate::flux::{ IFluxImportSource, IFluxImportSink, fluximport::FieldImp };
use crate::silo::{ U64, U32, U16, U8, USeg, Arr, Buff };

//---------------------------------------------------------------------------------------------------------------------------------
// Struct macros: generate IFluxExportSource and/or IFluxImportSource for named-field structs.

#[macro_export]
macro_rules! ImplFluxExportSource
{
    ( $struct_name:ident $( , $field:ident )* ) =>
    {
        impl $crate::flux::IFluxExportSource for $struct_name
        {
            fn	FetchFieldExp< 'a>( &'a self, field: &mut $crate::flux::FieldExp< 'a>)
            {
                let  	mut step = 0u32;
                let  	obj = self;
                *field = $crate::flux::FieldExp::Obj( Box::new( move |key, item| {
                    #[allow( unused_variables, unused_assignments)]
                    let  	mut _curr_step = 0u32;
                    $(
                        if step == _curr_step {
                            *key = stringify!( $field).to_string();
                            *item = $crate::flux::FieldExp::FluxSource( &obj.$field);
                            step += 1;
                            return true;
                        }
                        _curr_step += 1;
                    )*
                    false
                }));
            }
        }
    };
}

#[macro_export]
macro_rules! ImplFluxImportSource
{
    ( $struct_name:ident $( , $field:ident )* ) =>
    {
        impl $crate::flux::IFluxImportSource for $struct_name
        {
            fn	FetchFieldImp< 'a>( &'a mut self, field: &mut $crate::flux::FieldImp< 'a>)
            {
                let  	ptr = self as *mut Self;
                *field = $crate::flux::FieldImp::Obj( Box::new( move |key, item| {
                    let  	obj = unsafe { &mut *ptr };
                    let _ = &obj; let _ = &key; let _ = &item;
                    $(
                        if key == stringify!( $field) {
                            $crate::flux::IFluxImportSource::FetchFieldImp( &mut obj.$field, item);
                            return true;
                        }
                    )*
                    false
                }));
            }
        }
    };
}

#[macro_export]
macro_rules! ImplFluxSource
{
    ( $struct_name:ident $( , $field:ident )* ) =>
    {
        $crate::ImplFluxExportSource!( $struct_name $( , $field )* );
        $crate::ImplFluxImportSource!( $struct_name $( , $field )* );
    };
}

//---------------------------------------------------------------------------------------------------------------------------------
// Typed variant: adds a "Type" discriminator field for tagged structs.

#[macro_export]
macro_rules! ImplFluxSourceTyped
{
    ( $struct_name:ident, $type_name:literal $( , $field:ident )* ) =>
    {
        impl $crate::flux::IFluxExportSource for $struct_name
        {
            fn	FetchFieldExp< 'a>( &'a self, field: &mut $crate::flux::FieldExp< 'a>)
            {
                let  	mut step = 0u32;
                let  	obj = self;
                *field = $crate::flux::FieldExp::Obj( Box::new( move |key, item| {
                    if step == 0 {
                        *key = "Type".to_string();
                        *item = $crate::flux::FieldExp::Str( $type_name);
                        step += 1;
                        return true;
                    }
                    #[allow( unused_variables, unused_assignments)]
                    let  	mut _curr_step = 1u32;
                    $(
                        if step == _curr_step {
                            *key = stringify!( $field).to_string();
                            *item = $crate::flux::FieldExp::FluxSource( &obj.$field);
                            step += 1;
                            return true;
                        }
                        _curr_step += 1;
                    )*
                    false
                }));
            }
        }

        impl $crate::flux::IFluxImportSource for $struct_name
        {
            fn	FetchFieldImp< 'a>( &'a mut self, field: &mut $crate::flux::FieldImp< 'a>)
            {
                let  	ptr = self as *mut Self;
                *field = $crate::flux::FieldImp::Obj( Box::new( move |key, item| {
                    let  	obj = unsafe { &mut *ptr };
                    let _ = &obj; let _ = &key; let _ = &item;
                    if key == "Type" {
                        *item = $crate::flux::FieldImp::ExpectedType( $type_name);
                        return true;
                    }
                    $(
                        if key == stringify!( $field) {
                            $crate::flux::IFluxImportSource::FetchFieldImp( &mut obj.$field, item);
                            return true;
                        }
                    )*
                    false
                }));
            }
        }
    };
}

//---------------------------------------------------------------------------------------------------------------------------------
// Primitive leaf types.
// Types with a matching FieldExp/FieldImp variant (U64, f64) expose it directly by mutable ref.
// Narrower types (U8/U16/U32, f32) widen on export and route import through FluxSink with a cast.

macro_rules! ImplFluxPrimitive
{
    // Direct U64: self.0 is u64, FieldImp::U64 holds &mut U64
    ( $T:ty => U64 ) =>
    {
        impl $crate::flux::IFluxExportSource for $T
        {
            fn	FetchFieldExp< 'a>( &'a self, field: &mut $crate::flux::FieldExp< 'a>)
            {
                *field = $crate::flux::FieldExp::U64( $crate::silo::U64::From( self.0 as u64));
            }
        }
        impl $crate::flux::IFluxImportSource for $T
        {
            fn	FetchFieldImp< 'a>( &'a mut self, field: &mut $crate::flux::FieldImp< 'a>)
            {
                *field = $crate::flux::FieldImp::U64( self);
            }
        }
    };
    // Narrow uint: widens to U64 for export, receives via IFluxImportSink on import
    ( $T:ty => U64 via SINK ) =>
    {
        impl $crate::flux::IFluxExportSource for $T
        {
            fn	FetchFieldExp< 'a>( &'a self, field: &mut $crate::flux::FieldExp< 'a>)
            {
                *field = $crate::flux::FieldExp::U64( $crate::silo::U64::From( self.0 as u64));
            }
        }
        impl $crate::flux::IFluxImportSink for $T
        {
            fn	FromFieldImp( &mut self, field: $crate::flux::FieldImp) -> bool
            {
                if let $crate::flux::FieldImp::U64( val) = field { self.0 = val.0 as _; return true; }
                false
            }
        }
        impl $crate::flux::IFluxImportSource for $T
        {
            fn	FetchFieldImp< 'a>( &'a mut self, field: &mut $crate::flux::FieldImp< 'a>)
            {
                *field = $crate::flux::FieldImp::FluxSink( self);
            }
        }
    };
    // Direct f64: self is f64, FieldImp::F64 holds &mut f64
    ( $T:ty => F64 ) =>
    {
        impl $crate::flux::IFluxExportSource for $T
        {
            fn	FetchFieldExp< 'a>( &'a self, field: &mut $crate::flux::FieldExp< 'a>)
            {
                *field = $crate::flux::FieldExp::F64( *self as f64);
            }
        }
        impl $crate::flux::IFluxImportSource for $T
        {
            fn	FetchFieldImp< 'a>( &'a mut self, field: &mut $crate::flux::FieldImp< 'a>)
            {
                *field = $crate::flux::FieldImp::F64( self);
            }
        }
    };
    // Narrow float: widens to f64 for export, receives via IFluxImportSink on import
    ( $T:ty => F64 via SINK ) =>
    {
        impl $crate::flux::IFluxExportSource for $T
        {
            fn	FetchFieldExp< 'a>( &'a self, field: &mut $crate::flux::FieldExp< 'a>)
            {
                *field = $crate::flux::FieldExp::F64( *self as f64);
            }
        }
        impl $crate::flux::IFluxImportSink for $T
        {
            fn	FromFieldImp( &mut self, field: $crate::flux::FieldImp) -> bool
            {
                if let $crate::flux::FieldImp::F64( val) = field { *self = *val as _; return true; }
                false
            }
        }
        impl $crate::flux::IFluxImportSource for $T
        {
            fn	FetchFieldImp< 'a>( &'a mut self, field: &mut $crate::flux::FieldImp< 'a>)
            {
                *field = $crate::flux::FieldImp::FluxSink( self);
            }
        }
    };
}

ImplFluxPrimitive!( U64 => U64);
ImplFluxPrimitive!( U32 => U64 via SINK);
ImplFluxPrimitive!( U16 => U64 via SINK);
ImplFluxPrimitive!( U8  => U64 via SINK);
ImplFluxPrimitive!( f64 => F64);
ImplFluxPrimitive!( f32 => F64 via SINK);

//---------------------------------------------------------------------------------------------------------------------------------
// str / String

impl IFluxExportSource for String
{
    fn	FetchFieldExp< 'a>( &'a self, field: &mut FieldExp< 'a>)
    {
        *field = FieldExp::Str( self.as_str());
    }
}

impl IFluxImportSource for String
{
    fn	FetchFieldImp< 'a>( &'a mut self, field: &mut FieldImp< 'a>)
    {
        *field = FieldImp::String( self);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IFluxExportSource for str
{
    fn	FetchFieldExp< 'a>( &'a self, field: &mut FieldExp< 'a>)
    {
        *field = FieldExp::Str( self);
    }
}

impl< 'b> IFluxImportSource for &'b str
{
    fn	FetchFieldImp< 'a>( &'a mut self, field: &mut FieldImp< 'a>)
    {
        let ptr = self as *mut &'b str as *mut &'a str;
        *field = FieldImp::Str( unsafe { &mut *ptr } );
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
// USeg: two-field struct, uses macro (keys: "_First", "_Last")

crate::ImplFluxSource!( USeg, _First, _Last);

//---------------------------------------------------------------------------------------------------------------------------------
// Arr: fixed-size read-only slice wrapper

impl< 'a, T> IFluxExportSource for Arr< 'a, T>
where
    T: IFluxExportSource,
{
    fn	FetchFieldExp< 'b>( &'b self, field: &mut FieldExp< 'b>)
    {
        let  	mut idx = 0u32;
        let  	arr = *self;
        *field = FieldExp::Arr( Box::new( move |item| {
            if idx < arr._Size.0 {
                let  	elem = unsafe { &*arr._Ptr.as_ptr().add( idx as usize) };
                *item = FieldExp::FluxSource( elem);
                idx += 1;
                true
            } else {
                false
            }
        }));
    }
}

impl< 'a, T> IFluxImportSource for Arr< 'a, T>
where
    T: IFluxImportSource,
{
    fn	FetchFieldImp< 'b>( &'b mut self, field: &mut FieldImp< 'b>)
    {
        let  	mut idx = 0u32;
        let  	ptr = self as *mut Self;
        *field = FieldImp::Arr( Box::new( move |item| {
            let  	arr = unsafe { &mut *ptr };
            if idx < arr._Size.0 {
                let  	elem = unsafe { &mut *arr._Ptr.as_ptr().add( idx as usize) };
                *item = FieldImp::FluxSource( elem);
                idx += 1;
                true
            } else {
                assert!( idx < arr._Size.0, "Arr capacity exceeded during import. Use Buff instead.");
                false
            }
        }));
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
// Buff: growable heap array

impl< T> IFluxExportSource for Buff< T>
where
    T: IFluxExportSource,
{
    fn	FetchFieldExp< 'b>( &'b self, field: &mut FieldExp< 'b>)
    {
        let  	mut idx = 0usize;
        let  	ptr = self as *const Self;
        *field = FieldExp::Arr( Box::new( move |item| {
            let  	buff = unsafe { &*ptr };
            if idx < buff._Ptr.len() {
                let  	elem = unsafe { &*buff._Ptr.as_ptr().cast::< T>().add( idx) };
                *item = FieldExp::FluxSource( elem);
                idx += 1;
                true
            } else {
                false
            }
        }));
    }
}

impl< T> IFluxImportSource for Buff< T>
where
    T: IFluxImportSource + Default,
{
    fn	FetchFieldImp< 'b>( &'b mut self, field: &mut FieldImp< 'b>)
    {
        let  	mut idx = 0usize;
        let  	ptr = self as *mut Self;
        *field = FieldImp::Arr( Box::new( move |item| {
            let  	buff = unsafe { &mut *ptr };
            if idx >= buff._Ptr.len() {
                buff.Push( T::default());
            }
            let  	elem = unsafe { &mut *buff._Ptr.as_ptr().cast::< T>().add( idx) };
            *item = FieldImp::FluxSource( elem);
            idx += 1;
            true
        }));
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
