use std::fmt::{Debug, Formatter, Result as FmtResult};

const SYS_MACHDEP: i64 = 0x80000000;
const MACHDEP_SET_CTHREAD_SELF: u64 = 2;
const MACHDEP_GET_CTHREAD_SELF: u64 = 3;

#[repr(u64)]
#[derive(Clone, Copy)]
pub enum MachDep {
    SetCthreadSelf(u64) = MACHDEP_SET_CTHREAD_SELF,
    GetCthreadSelf = MACHDEP_GET_CTHREAD_SELF,
    Unknown(u64),
}

impl MachDep {
    #[inline]
    pub const fn decode(id: u64, args: &[u64; 9]) -> Self {
        match id {
            MACHDEP_SET_CTHREAD_SELF => Self::SetCthreadSelf(args[0]),
            MACHDEP_GET_CTHREAD_SELF => Self::GetCthreadSelf,
            _ => Self::Unknown(id),
        }
    }

    #[inline]
    pub const fn is_machdep_trap(num: i64) -> bool {
        num == SYS_MACHDEP
    }
}

impl Debug for MachDep {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::SetCthreadSelf(tsd) => write!(f, "set_cthread_self(tsd=0x{tsd:x})"),
            Self::GetCthreadSelf => write!(f, "get_cthread_self()"),
            Self::Unknown(id) => write!(f, "Unknown({id})"),
        }
    }
}
