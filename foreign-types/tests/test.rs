use foreign_types::foreign_type;

mod foo_sys {
    pub enum FOO {}

    pub unsafe extern "C" fn foo_drop(_: *mut FOO) {}
    pub unsafe extern "C" fn foo_clone(ptr: *mut FOO) -> *mut FOO {
        ptr
    }

    pub unsafe extern "C" fn foo_drop_requiring_cast(_: *mut ()) {}
    pub unsafe extern "C" fn foo_clone_requiring_cast(ptr: *mut ()) -> *mut () {
        ptr
    }
}

foreign_type! {
    pub unsafe type Foo<'a, T>: Sync + Send {
        type CType = foo_sys::FOO;
        type PhantomData = &'a T;
        fn drop = foo_sys::foo_drop;
        fn clone = foo_sys::foo_clone;
    }

    pub unsafe type FooNoClone {
        type CType = foo_sys::FOO;
        fn drop = foo_sys::foo_drop;
    }

    pub unsafe type FooClosure {
        type CType = foo_sys::FOO;
        fn drop = |p| foo_sys::foo_drop(p);
        fn clone = |p| foo_sys::foo_clone(p);
    }

    pub unsafe type FooClosureCast {
        type CType = foo_sys::FOO;
        fn drop = |p| foo_sys::foo_drop_requiring_cast(p as _);
        fn clone = |p| foo_sys::foo_clone_requiring_cast(p as _) as _;
    }
}
