pub enum Shard<'a> {
    A(std::marker::PhantomData<&'a mut ()>),
}
fn test<'node>(shard: &Shard<'node>) -> Option<&'static Shard<'static>> {
    unsafe { Some(&*(shard as *const Shard<'node> as *const Shard<'static>)) }
}
