use mach2::{
    kern_return::{KERN_INVALID_ARGUMENT, KERN_NOT_SUPPORTED, KERN_SUCCESS, kern_return_t},
    message::{
        MACH_MSG_TYPE_MOVE_SEND_ONCE, MACH_MSGH_BITS, mach_msg_body_t, mach_msg_header_t,
        mach_msg_id_t, mach_msg_port_descriptor_t,
    },
    ndr::NDR_record_t,
    port::{MACH_PORT_NULL, mach_port_t},
    vm::mach_vm_map,
    vm_inherit::VM_INHERIT_DEFAULT,
    vm_prot::{VM_PROT_ALL, VM_PROT_ALLEXEC, VM_PROT_EXECUTE, VM_PROT_READ, VM_PROT_WRITE},
    vm_statistics::VM_FLAGS_ANYWHERE,
};

use crate::{
    aarch64::{
        cpu::Cpu,
        syscall::mach::{
            ARG_mach_msg_overwrite_trap, ARG_mach_msg_trap, ARG_mach_msg2_trap, mach_msg_option64_t,
        },
        virtos::{
            mem::{VmKind, VmMap},
            task::TASK_SELF,
        },
    },
    mem::Protection,
    utils::{ptr::Uintptr, size::align_to_page},
};

const MACH_VM_ALLOCATE: u32 = 4800;
const MACH_VM_DEALLOCATE: u32 = 4801;
const MACH_VM_PROTECT: u32 = 4802;
const MACH_VM_MAP: u32 = 4811;
const MACH_VM_REMAP: u32 = 4813;
const MACH_VM_REMAP_NEW: u32 = 4821;
const MACH_VM_DEFERRED_RECLAMATION_BUFFER_ALLOCATE: u32 = 4822;
const MACH_VM_DEFERRED_RECLAMATION_BUFFER_FLUSH: u32 = 4823;
const MACH_VM_DEFERRED_RECLAMATION_BUFFER_RESIZE: u32 = 4826;
const MACH_VM_DEFERRED_RECLAMATION_BUFFER_QUERY: u32 = 4828;

impl ARG_mach_msg2_trap {
    #[inline]
    const fn req_id(&self) -> u32 {
        self.msgh_voucher_and_id.msb() as u32
    }

    #[inline]
    const fn recv_size(&self) -> usize {
        self.rcv_size_and_priority.lsb() as usize
    }

    #[inline]
    const fn send_size(&self) -> usize {
        self.msgh_bits_and_send_size.msb() as usize
    }

    #[inline]
    const fn local_port(&self) -> mach_port_t {
        self.msgh_remote_and_local_port.msb() as mach_port_t
    }
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct mig_reply_error_t {
    head: mach_msg_header_t,
    ndr: NDR_record_t,
    status: kern_return_t,
}

impl mig_reply_error_t {
    fn reply(port: &Responder, status: kern_return_t) {
        port.reply(Self {
            ndr: unsafe { mach2::ndr::NDR_record },
            head: port.header::<Self>(),
            status,
        });
    }
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct mach_vm_map_request {
    head: mach_msg_header_t,
    body: mach_msg_body_t,
    object: mach_msg_port_descriptor_t,
    ndr: NDR_record_t,
    address: Uintptr,
    size: u64,
    mask: u64,
    flags: i32,
    offset: u64,
    copy: i32,
    cur_prot: i32,
    max_prot: i32,
    inheritance: u32,
}

impl mach_vm_map_request {
    #[inline]
    const fn hack_is_xzm_region_alloc(&self) -> bool {
        self.address.addr() >= 0x400000000
            && self.size == 0x600000000
            && self.mask == 0
            && self.flags == 0
            && self.offset == 0
            && self.copy == 0
            && self.cur_prot == 0
            && self.max_prot == 0
            && self.inheritance == VM_INHERIT_DEFAULT
    }
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct mach_vm_allocate_or_map_reply {
    head: mach_msg_header_t,
    ndr: NDR_record_t,
    status: kern_return_t,
    address: Uintptr,
}

impl mach_vm_allocate_or_map_reply {
    fn reply(port: &Responder, addr: Uintptr) {
        port.reply(Self {
            ndr: unsafe { mach2::ndr::NDR_record },
            head: port.header::<Self>(),
            status: KERN_SUCCESS,
            address: addr,
        });
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct mach_msg_vector_t {
    msgv_send_data: Uintptr,
    msgv_recv_data: Uintptr,
    msgv_send_size: usize,
    msgv_recv_size: usize,
}

struct Responder {
    size: usize,
    data: Uintptr,
    port: mach_port_t,
    resp: mach_msg_id_t,
}

impl Responder {
    #[inline]
    const fn new(id: u32, port: mach_port_t, buffer: mach_msg_vector_t) -> Self {
        Self {
            port,
            size: buffer.msgv_recv_size,
            data: buffer.msgv_recv_data,
            resp: (id as mach_msg_id_t) + 100,
        }
    }
}

impl Responder {
    #[inline]
    fn reply<R>(&self, data: R) {
        assert!(self.size >= std::mem::size_of::<R>());
        self.data.write(data);
    }
}

impl Responder {
    #[inline]
    fn header<T>(&self) -> mach_msg_header_t {
        mach_msg_header_t {
            msgh_bits: MACH_MSGH_BITS(MACH_MSG_TYPE_MOVE_SEND_ONCE, 0),
            msgh_size: std::mem::size_of::<T>() as u32,
            msgh_remote_port: 0,
            msgh_local_port: self.port,
            msgh_voucher_port: 0,
            msgh_id: self.resp,
        }
    }
}

#[inline]
fn parse_message<T: Copy>(args: &ARG_mach_msg2_trap) -> Option<(T, Responder)> {
    let is_vectored_msg = {
        args.options
            .contains(mach_msg_option64_t::MACH64_MSG_VECTOR)
    };
    let buffer = {
        if !is_vectored_msg {
            mach_msg_vector_t {
                msgv_send_data: Uintptr::from(args.data),
                msgv_recv_data: Uintptr::from(args.data),
                msgv_send_size: args.send_size(),
                msgv_recv_size: args.recv_size(),
            }
        } else if args.send_size() != 0 {
            unsafe { *(args.data as *const mach_msg_vector_t) }
        } else {
            return None;
        }
    };
    if buffer.msgv_send_size == std::mem::size_of::<T>() {
        Some((
            buffer.msgv_send_data.read(),
            Responder::new(args.req_id(), args.local_port(), buffer),
        ))
    } else {
        None
    }
}

fn handle_mach_vm_map(_cpu: &Cpu, mut req: mach_vm_map_request, port: Responder) -> kern_return_t {
    macro_rules! set_prot {
        ($prot:ident, $flag:ident, $name:ident) => {
            if req.$prot & $flag != 0 {
                $prot |= Protection::$name;
            }
        };
    }

    /* check if we have enough space for response */
    if port.size < std::mem::size_of::<mach_vm_allocate_or_map_reply>() {
        return KERN_INVALID_ARGUMENT;
    }

    /* make a copy of the desired protection */
    let mut addr = req.address.as_u64();
    let mut cur_prot = Protection::NONE;
    let mut max_prot = Protection::NONE;

    /* convert current protection flags */
    set_prot!(cur_prot, VM_PROT_READ, READ);
    set_prot!(cur_prot, VM_PROT_WRITE, WRITE);
    set_prot!(cur_prot, VM_PROT_EXECUTE, EXEC);

    /* convert max protection flags */
    set_prot!(max_prot, VM_PROT_READ, READ);
    set_prot!(max_prot, VM_PROT_WRITE, WRITE);
    set_prot!(max_prot, VM_PROT_EXECUTE, EXEC);

    /* check if it's an object map */
    if req.object.name != 0 {
        unimplemented!("mach_vm_map() message with non-zero port");
    }

    // FIXME: this is a hack to get around the address space reservation for xzone malloc, to
    // implement this correctly, we may need to implement full virtual address translation, and
    // perform memory copy and / or address translation on every syscall boundary, rather than
    // identity mapping (PA=IPA).
    if req.hack_is_xzm_region_alloc() {
        addr = 0;
        req.flags |= VM_FLAGS_ANYWHERE;
    }

    /* perform the actual memory map */
    let status = unsafe {
        mach_vm_map(
            *TASK_SELF,
            &raw mut addr,
            req.size,
            req.mask,
            req.flags,
            MACH_PORT_NULL,
            0,
            0,
            req.cur_prot & !VM_PROT_ALLEXEC,
            VM_PROT_ALL,
            VM_INHERIT_DEFAULT,
        )
    };

    /* check if the syscall is successful */
    if status != KERN_SUCCESS {
        mig_reply_error_t::reply(&port, status);
        return KERN_SUCCESS;
    }

    /* insert into page table */
    VmMap::map(
        VmKind::Regular,
        Uintptr::from(addr),
        align_to_page(req.size as usize),
        cur_prot,
        max_prot,
        false,
    );

    /* send the response */
    mach_vm_allocate_or_map_reply::reply(&port, addr.into());
    KERN_SUCCESS
}

#[inline]
pub fn mach_msg_trap(_cpu: &Cpu, _args: ARG_mach_msg_trap) -> kern_return_t {
    KERN_NOT_SUPPORTED
}

#[inline]
pub fn mach_msg_overwrite_trap(_cpu: &Cpu, _args: ARG_mach_msg_overwrite_trap) -> kern_return_t {
    KERN_NOT_SUPPORTED
}

pub fn mach_msg2_trap(cpu: &Cpu, mut args: ARG_mach_msg2_trap) -> kern_return_t {
    match args.req_id() {
        MACH_VM_ALLOCATE => {
            unimplemented!("mach_vm_allocate() through mach_msg2_trap()");
        }
        MACH_VM_DEALLOCATE => {
            unimplemented!("mach_vm_deallocate() through mach_msg2_trap()");
        }
        MACH_VM_PROTECT => {
            unimplemented!("mach_vm_protect() through mach_msg2_trap()");
        }
        MACH_VM_MAP => {
            if !args.options.contains(mach_msg_option64_t::MACH64_RCV_MSG) {
                KERN_INVALID_ARGUMENT
            } else if let Some((req, port)) = parse_message(&args) {
                handle_mach_vm_map(cpu, req, port)
            } else {
                KERN_INVALID_ARGUMENT
            }
        }
        MACH_VM_REMAP => {
            unimplemented!("mach_vm_remap() through mach_msg2_trap()");
        }
        MACH_VM_REMAP_NEW => {
            unimplemented!("mach_vm_remap_new() through mach_msg2_trap()");
        }
        // TODO: implement these
        MACH_VM_DEFERRED_RECLAMATION_BUFFER_ALLOCATE => KERN_NOT_SUPPORTED,
        MACH_VM_DEFERRED_RECLAMATION_BUFFER_FLUSH => KERN_NOT_SUPPORTED,
        MACH_VM_DEFERRED_RECLAMATION_BUFFER_RESIZE => KERN_NOT_SUPPORTED,
        MACH_VM_DEFERRED_RECLAMATION_BUFFER_QUERY => KERN_NOT_SUPPORTED,
        _ => {
            unsafe {
                std::arch::asm!(
                    "mov x16, #-47",
                    "svc #0x80",
                    inout("x0") args.data,
                    in("x1") args.options.bits(),
                    in("x2") args.msgh_bits_and_send_size.value(),
                    in("x3") args.msgh_remote_and_local_port.value(),
                    in("x4") args.msgh_voucher_and_id.value(),
                    in("x5") args.desc_count_and_rcv_name.value(),
                    in("x6") args.rcv_size_and_priority.value(),
                    in("x7") args.timeout,
                );
            };
            args.data as kern_return_t
        }
    }
}
