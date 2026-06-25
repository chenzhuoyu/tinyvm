#[inline]
pub fn align_to_page(size: usize) -> usize {
    unsafe { (size + libc::vm_page_size - 1) & !(libc::vm_page_size - 1) }
}

#[inline]
pub fn is_page_aligned(value: usize) -> bool {
    unsafe { value.is_multiple_of(libc::vm_page_size) }
}
