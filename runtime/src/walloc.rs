pub struct WBox {}

impl Drop for WBox {
    fn drop(&mut self) {}
}

/*

    // TODO: Make an allocation module?
    fn alloc_value<T: Sized>(caller: &Caller, value: &T) -> (u32, std::alloc::Layout) {
        let mem = caller.get_export("memory").unwrap().into_memory().unwrap();
        let malloc = caller.get_export("__alloc").unwrap().into_func().unwrap();
        let malloc = malloc.typed::<(i32, u32), u32>().unwrap();

        let layout = std::alloc::Layout::new::<T>();

        let data_ptr = malloc
            .call((layout.size() as _, layout.align() as _))
            .unwrap();

        unsafe {
            let mem = mem.data_ptr().add(data_ptr as _) as *mut T;
            std::ptr::copy(value, mem, 1);
        }

        return (data_ptr, layout);
    }

    fn alloc_vec<T: Sized>(caller: &Caller, value: &[T]) -> (u32, std::alloc::Layout) {
        let mem = caller.get_export("memory").unwrap().into_memory().unwrap();
        let malloc = caller.get_export("__alloc").unwrap().into_func().unwrap();
        let malloc = malloc.typed::<(i32, u32), u32>().unwrap();

        let layout = std::alloc::Layout::array::<T>(value.len()).unwrap();

        let data_ptr = malloc
            .call((layout.size() as _, layout.align() as _))
            .unwrap();

        unsafe {
            let mem = mem.data_ptr().add(data_ptr as _) as *mut T;
            std::ptr::copy(value.as_ptr(), mem, value.len());
        }

        return (data_ptr, layout);
    }

*/
