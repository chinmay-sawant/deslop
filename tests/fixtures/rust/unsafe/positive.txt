unsafe fn unchecked(values: &[u8], index: usize) -> u8 {
    *values.get_unchecked(index)
}

unsafe fn raw_parts(ptr: *const u8, len: usize) -> &'static [u8] {
    std::slice::from_raw_parts(ptr, len)
}

unsafe fn set_vector_len(values: &mut Vec<u8>) {
    values.set_len(8);
}

unsafe fn init_value(value: std::mem::MaybeUninit<String>) -> String {
    value.assume_init()
}

unsafe fn cast_pointer(ptr: *const u8) -> *const u32 {
    ptr as *const u32
}

unsafe fn transmute_value(raw: u32) -> f32 {
    std::mem::transmute(raw)
}