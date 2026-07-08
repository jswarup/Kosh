sed -i 's/Option< \*const DynIWork< '"'"'static>>/Option<*mut core::ffi::c_void>/g' src/stalks/node.rs
sed -i 's/Action( Box<DynIWork< '"'"'static>> )/Action( crate::stalks::node::ActionFn )/g' src/stalks/node.rs
