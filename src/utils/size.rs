use crate::paging::PAGE_SIZE;

#[inline(always)]
pub fn align_to_page(size: usize) -> usize {
    (size + PAGE_SIZE - 1) & !(PAGE_SIZE - 1)
}

#[inline(always)]
pub fn is_page_aligned(value: usize) -> bool {
    value.is_multiple_of(PAGE_SIZE)
}
