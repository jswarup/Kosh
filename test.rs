pub trait Parser {}
pub struct MyParser;
impl Parser for MyParser {}

pub fn main() {
    let mut b: Box<dyn FnMut(&mut MyParser)> = Box::new(|p| {});
    let p_box = &mut b as *mut Box<dyn FnMut(&mut MyParser)> as *mut core::ffi::c_void;
    
    let p_box_back = p_box as *mut Box<dyn FnMut(&mut MyParser)>;
    let func = unsafe { &mut *p_box_back };
    let mut parser = MyParser;
    func(&mut parser);
}
