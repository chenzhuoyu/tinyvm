use crate::aarch64::regs::PSTATE_C;

#[inline]
pub fn syscall<const N: usize>(num: i64, argv: [u64; N]) -> (u64, bool)
where
    [(); 9 - N]:,
{
    let mut nzcv = 0;
    let mut args = [0; 9];
    args[..N].copy_from_slice(&argv);
    syscall_inplace(num, &mut args, &mut nzcv);
    (args[0], nzcv & PSTATE_C == 0)
}

#[inline]
pub fn syscall_inplace(num: i64, args: &mut [u64; 9], nzcv: &mut u64) {
    unsafe {
        std::arch::asm!(
            "svc #0x80",
            "mrs {}, nzcv",
            out(reg) *nzcv,
            inout("x0") args[0],
            in("x1") args[1],
            in("x2") args[2],
            in("x3") args[3],
            in("x4") args[4],
            in("x5") args[5],
            in("x6") args[6],
            in("x7") args[7],
            in("x8") args[8],
            in("x16") num,
        );
    }
}
