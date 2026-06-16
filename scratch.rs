pub struct U32(pub u32);

pub trait IAccess< 'a, T: 'a + ?Sized> {
    fn	Size( &self) -> U32;
    fn	At( &self, k: U32) -> &'a T;
}

pub struct MyTree { data: std::boxed::Box<u32> }

impl<'b> IAccess<'b, u32> for &'b MyTree {
    fn Size(&self) -> U32 { U32(1) }
    fn At(&self, idx: U32) -> &'b u32 {
        &(*self).data
    }
}

fn main() {
    let t = MyTree { data: std::boxed::Box::new(42) };
    let access: &dyn IAccess<'_, u32> = &&t;
    println!("{}", access.At(U32(0)));
}
