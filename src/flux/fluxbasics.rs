//-- fluxbasics.rs -----------------------------------------------------------------------------------------------------------------------

use crate::flux::{ IFluxExportSource, fluxexport::FieldExp };
use crate::flux::{ IFluxImportSource, fluximport::FieldImp };
use crate::silo::{ U64, U32, U16, U8, USeg, Arr, Buff };

//---------------------------------------------------------------------------------------------------------------------------------

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

#[macro_export]
macro_rules! ImplFluxSourceUInt
{
    ( $( $T:ty ),+ ) =>
    {
        $(
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
                    *field = $crate::flux::FieldImp::FluxSource( self);
                }
            }
        )+
    };
}

//---------------------------------------------------------------------------------------------------------------------------------

#[macro_export]
macro_rules! ImplFluxSourceFloat
{
    ( $( $T:ty ),+ ) =>
    {
        $(
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
                    *field = $crate::flux::FieldImp::FluxSource( self);
                }
            }
        )+
    };
}

//---------------------------------------------------------------------------------------------------------------------------------

ImplFluxSourceUInt!( U8, U16, U32, U64);
ImplFluxSourceFloat!( f32, f64);

//---------------------------------------------------------------------------------------------------------------------------------

impl IFluxExportSource for String {
    fn FetchFieldExp<'a>(&'a self, field: &mut FieldExp<'a>) {
        *field = FieldExp::Str(self.as_str());
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

impl IFluxExportSource for str {
    fn FetchFieldExp<'a>(&'a self, field: &mut FieldExp<'a>) {
        *field = FieldExp::Str(self);
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

impl IFluxExportSource for USeg
{
    fn	FetchFieldExp< 'a>( &'a self, field: &mut FieldExp< 'a>)
    {
        let  	mut step = 0u32;
        let  	uSeg = self;
        *field = FieldExp::Obj( Box::new( move |key, item| {
            if step == 0 {
                *key = "First".to_string();
                *item = FieldExp::FluxSource( &uSeg._First);
                step += 1;
                return true;
            }
            if step == 1 {
                *key = "Last".to_string();
                *item = FieldExp::FluxSource( &uSeg._Last);
                step += 1;
                return true;
            }
            false
        }));
    }
}

impl IFluxImportSource for USeg
{
    fn	FetchFieldImp< 'a>( &'a mut self, field: &mut FieldImp< 'a>)
    {
        let  	ptr = self as *mut Self;
        *field = FieldImp::Obj( Box::new( move |key, item| {
            let  	obj = unsafe { &mut *ptr };
            if key == "First" {
                *item = FieldImp::FluxSource( &mut obj._First);
                return true;
            }
            if key == "Last" {
                *item = FieldImp::FluxSource( &mut obj._Last);
                return true;
            }
            false
        }));
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl<'a, T> IFluxExportSource for Arr<'a, T>
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

impl<'a, T> IFluxImportSource for Arr<'a, T>
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

impl<T> IFluxExportSource for Buff<T>
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
                let  	elem = unsafe { &*buff._Ptr.as_ptr().cast::<T>().add( idx) };
                *item = FieldExp::FluxSource( elem);
                idx += 1;
                true
            } else {
                false
            }
        }));
    }
}

impl<T> IFluxImportSource for Buff<T>
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
            let  	elem = unsafe { &mut *buff._Ptr.as_ptr().cast::<T>().add( idx) };
            *item = FieldImp::FluxSource( elem);
            idx += 1;
            true
        }));
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
