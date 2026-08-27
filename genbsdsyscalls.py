#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import dataclasses
import re
import subprocess

PRELUDE = r'''#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::fmt::{Debug, Formatter, Result as FmtResult};

use crate::{
    macros::define_bit_field,
    utils::{ptr::VMA, str::Sz},
};

pub type au_asflgs_t = u64;
pub type au_asid_t = libc::pid_t;
pub type au_id_t = libc::uid_t;
pub type guardid_t = u64;

pub const NFS_MAX_FH_SIZE: usize = NFSV4_MAX_FH_SIZE;
pub const NFSV4_MAX_FH_SIZE: usize = 128;
pub const NFSV3_MAX_FH_SIZE: usize = 64;
pub const NFSV2_MAX_FH_SIZE: usize = 32;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct fhandle_t {
	pub fh_len: u32,
	pub fh_data: [u8; NFS_MAX_FH_SIZE],
}

pub const GRAFTDMG_SECURE_BOOT_CRYPTEX_ARGS_VERSION: u32 = 1;
pub const MAX_GRAFT_ARGS_SIZE: usize = 512;

// Flag values for secure_boot_cryptex_args_t.sbc_flags
pub const SBC_PRESERVE_MOUNT          : u64 = 0x0001;  // Preserve underlying mount until shutdown
pub const SBC_ALTERNATE_SHARED_REGION : u64 = 0x0002;  // Binaries within should use alternate shared region
pub const SBC_SYSTEM_CONTENT          : u64 = 0x0004;  // Cryptex contains system content
pub const SBC_PANIC_ON_AUTHFAIL       : u64 = 0x0008;  // On failure to authenticate, panic
pub const SBC_STRICT_AUTH             : u64 = 0x0010;  // Strict authentication mode
pub const SBC_PRESERVE_GRAFT          : u64 = 0x0020;  // Preserve graft itself until unmount

// Flag values for ungraftdmg
pub const UNGRAFTDMG_NOFORCE: u64 = 0x0000000000000002; // Disallow ungraft if a non-dir vnode inside the graft is in use

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct secure_boot_cryptex_args_t {
	pub sbc_version: u32,
	pub sbc_4cc: u32,
	pub sbc_authentic_manifest_fd: i32,
	pub sbc_user_manifest_fd: i32,
	pub sbc_payload_fd: i32,
	pub sbc_flags: u64,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union graftdmg_args_un {
	pub max_size: [u8; MAX_GRAFT_ARGS_SIZE],
	pub sbc_args: secure_boot_cryptex_args_t,
}

impl Debug for graftdmg_args_un {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        unsafe {
            f.debug_struct("graftdmg_args_un")
                .field_with("max_size", |f| write!(f, "[...]"))
                .field("sbc_args", &self.sbc_args)
                .finish()
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union semun_t {
    pub val: i32,
    pub buf: *mut libc::semid_ds,
    pub array: *mut u16,
    pub value: u64,
}

impl semun_t {
    #[inline]
    fn from_u64(value: u64) -> Self {
        Self { value }
    }
}

impl Debug for semun_t {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        unsafe {
            f.debug_struct("semun_t")
                .field("val", &self.val)
                .field("buf", &self.buf)
                .field("array", &self.array)
                .finish()
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct shared_mapping_np {
	pub sms_address: VMA,
	pub sms_size: libc::mach_vm_size_t,
	pub sms_file_offset: libc::mach_vm_offset_t,
	pub sms_slide_size: usize,
	pub sms_slide_start: VMA,
	pub sms_max_prot: libc::vm_prot_t,
	pub sms_init_prot: libc::vm_prot_t,
}

pub type posix_spawn_port_actions_t = *mut libc::c_void; // opqaue pointer
pub type posix_spawn_mac_policy_extensions_t = *mut libc::c_void; // opqaue pointer
pub type posix_spawn_coalition_info = libc::c_void; // opqaue struct
pub type posix_spawn_persona_info = libc::c_void; // opqaue struct
pub type posix_spawn_posix_cred_info = libc::c_void; // opqaue struct

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct posix_spawn_args_desc {
	pub attr_size: usize,
	pub attrp: libc::posix_spawnattr_t,
	pub file_actions_size: usize,
	pub file_actions: libc::posix_spawn_file_actions_t,
	pub port_actions_size: usize,
	pub port_actions: posix_spawn_port_actions_t,
	pub mac_extensions_size: usize,
	pub mac_extensions: posix_spawn_mac_policy_extensions_t,
	pub coal_info_size: usize,
	pub coal_info: *mut posix_spawn_coalition_info,
	pub persona_info_size: usize,
	pub persona_info: *mut posix_spawn_persona_info,
	pub posix_cred_info_size: usize,
	pub posix_cred_info: *mut posix_spawn_posix_cred_info,
	pub subsystem_root_path_size: usize,
	pub subsystem_root_path: Sz,
	pub conclave_id_size: usize,
	pub conclave_id: Sz,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct au_mask_t {
	pub am_success: u32,
	pub am_failure: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct au_tid_addr_t {
	pub at_port: libc::dev_t,
	pub at_type: u32,
	pub at_addr: [u32; 4],
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct auditinfo_addr {
	pub ai_auid: au_id_t,
	pub ai_mask: au_mask_t,
	pub ai_termid: au_tid_addr_t,
	pub ai_asid: au_asid_t,
	pub ai_flags: au_asflgs_t,
}

pub type ch_info = libc::c_void; // opqaue struct
pub type ch_init = libc::c_void; // opqaue struct

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct fssearchblock {
	pub returnattrs: *mut libc::attrlist,
	pub returnbuffer: *mut libc::c_void,
	pub returnbuffersize: usize,
	pub maxmatches: usize,
	pub timelimit: libc::timeval,
	pub searchparams1: *mut libc::c_void,
	pub sizeofsearchparams1: usize,
	pub searchparams2: *mut libc::c_void,
	pub sizeofsearchparams2: usize,
	pub searchattrs: libc::attrlist,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct kevent_qos_s {
	pub ident: u64,
	pub filter: i16,
	pub flags: u16,
	pub qos: i32,
	pub udata: u64,
	pub fflags: u32,
	pub xflags: u32,
	pub data: i64,
	pub ext: [u64; 4],
}

pub const NGROUPS: usize = 16;
pub const MAXLOGNAME: usize = 255;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct kpersona_info {
	pub persona_info_version: u32,
	pub persona_id: libc::uid_t,
	pub persona_type: i32,
	pub persona_gid: libc::gid_t,
	pub persona_ngroups: u32,
	pub persona_groups: [libc::gid_t; NGROUPS],
	pub persona_gmuid: libc::uid_t,
	pub persona_name: [i8; MAXLOGNAME + 1],
	pub persona_uid: libc::uid_t,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct mac {
	pub m_buflen: usize,
	pub m_string: Sz,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct msghdr_x {
	pub msg_name: *mut libc::c_void,
	pub msg_namelen: libc::socklen_t,
	pub msg_iov: *mut libc::iovec,
	pub msg_iovlen: i32,
	pub msg_control: *mut libc::c_void,
	pub msg_controllen: libc::socklen_t,
	pub msg_flags: i32,
	pub msg_datalen: usize,
}

pub type msqid_ds = libc::c_void; // opqaue struct
pub type necp_aggregate_result = libc::c_void; // opqaue struct

define_bit_field! {
    pub struct net_qos_param_flags : u32 {
	    nq_use_expensive: 1,
	    nq_uplink: 1,
	    nq_use_constrained: 1,
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct net_qos_param {
	pub nq_transfer_size: usize,
    pub nq_flags: net_qos_param_flags,
	pub nq_unused: u32,
}

pub type nxctl_init = libc::c_void; // opaque struct
pub type nxprov_reg = libc::c_void; // opaque struct

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct searchstate {
	pub ss_union_flags: u32,
	pub ss_union_layer: u32,
	pub ss_fsstate: [u8; 548],
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct shared_file_np {
	pub sf_fd: i32,
	pub sf_mappings_count: u32,
	pub sf_slide: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct sigaltstack {
	pub ss_sp: *mut libc::c_void,
	pub ss_size: usize,
	pub ss_flags: i32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct sigvec {
	pub sv_handler: Option<unsafe extern "C" fn(i32)>,
	pub sv_mask: i32,
	pub sv_flags: i32,
}

trait Arg {
    fn decode(args: &[u64; 9]) -> Self;
}

#[inline]
fn mk_uuid(lsb: u64, msb: u64) -> libc::uuid_t {
    let mut uuid = [0u8; 16];
    uuid[..8].copy_from_slice(&lsb.to_le_bytes());
    uuid[8..].copy_from_slice(&msb.to_le_bytes());
    uuid
}
'''

BSD_IMPL_PRELUDE = r'''
use crate::aarch64::{
    syscall::{Syscall, bsd::BsdSyscall},
    virtos::syscall::bsd,
};

impl Syscall<'_> {
    pub(super) fn dispatch_bsd(&mut self, bsd: BsdSyscall) {
        macro_rules! handle_syscall {
            ($($name:ident $(($($args:tt)*))?),+ $(,)?) => {
                match bsd {
                    $(
                        handle_syscall!(@CASE $name [$(($($args)*))?]) => {
                            handle_syscall!(@IMPL $name [$(($($args)*))?]);
                        }
                    )*
                    BsdSyscall::Unknown(..) => {
                        self.bsd_error(libc::ENOSYS);
                    },
                }
            };
            (@CALL $name:ident $(, $args:expr)?) =>  {
                match bsd::$name(self.cpu $(, $args)?) {
                    Err(err) => self.bsd_error(err.raw_os_error().unwrap_or(-1)),
                    Ok(ret) => self.bsd_result(ret),
                }
            };
            (@CASE $name:ident [($args:ident)]) => { BsdSyscall::$name($args) };
            (@CASE $name:ident [(..)]) => { BsdSyscall::$name(..) };
            (@CASE $name:ident [()]) => { BsdSyscall::$name };
            (@CASE $name:ident []) => { BsdSyscall::$name };
            (@IMPL $name:ident [($args:ident)]) => { handle_syscall!(@CALL $name, $args) };
            (@IMPL $name:ident [(..)]) => { self.forward() };
            (@IMPL $name:ident [()]) => { handle_syscall!(@CALL $name) };
            (@IMPL $name:ident []) => { self.forward() };
        }
        handle_syscall! {
'''

TYPE_MAP = {
    'au_asid_t'                       : 'au_asid_t',
    'au_id_t'                         : 'au_id_t',
    'caddr_t'                         : 'VMA',
    'caddr_ut'                        : 'VMA',
    'char'                            : 'i8',
    'fhandle_t'                       : 'fhandle_t',
    'gid_t'                           : 'libc::gid_t',
    'graftdmg_args_un'                : 'graftdmg_args_un',
    'guardid_t'                       : 'guardid_t',
    'id_t'                            : 'libc::id_t',
    'idtype_t'                        : 'libc::idtype_t',
    'int'                             : 'i32',
    'int32_t'                         : 'i32',
    'int64_t'                         : 'i64',
    'key_t'                           : 'libc::key_t',
    'long'                            : 'i64',
    'mach_port_name_t'                : 'u32',
    'off_t'                           : 'libc::off_t',
    'pid_t'                           : 'libc::pid_t',
    'sa_endpoints_t'                  : 'libc::sa_endpoints_t',
    'sae_associd_t'                   : 'libc::sae_associd_t',
    'sae_connid_t'                    : 'libc::sae_connid_t',
    'sem_t'                           : 'libc::sem_t',
    'semun_t'                         : 'semun_t',
    'shared_file_mapping_slide_np_ut' : 'shared_mapping_np',
    'siginfo_t'                       : 'libc::siginfo_t',
    'sigset_t'                        : 'libc::sigset_t',
    'size_t'                          : 'usize',
    'size_ut'                         : 'usize',
    'socklen_t'                       : 'libc::socklen_t',
    'struct __sigaction'              : 'libc::sigaction',
    'struct _posix_spawn_args_desc'   : 'posix_spawn_args_desc',
    'struct attrlist'                 : 'libc::attrlist',
    'struct auditinfo_addr'           : 'auditinfo_addr',
    'struct ch_info'                  : 'ch_info',
    'struct ch_init'                  : 'ch_init',
    'struct fhandle'                  : 'fhandle_t',
    'struct fssearchblock'            : 'fssearchblock',
    'struct iovec'                    : 'libc::iovec',
    'struct itimerval'                : 'libc::itimerval',
    'struct kevent_qos_s'             : 'kevent_qos_s',
    'struct kevent'                   : 'libc::kevent',
    'struct kevent64_s'               : 'libc::kevent64_s',
    'struct kpersona_info'            : 'kpersona_info',
    'struct mac'                      : 'mac',
    'struct msghdr_x'                 : 'msghdr_x',
    'struct msghdr'                   : 'libc::msghdr',
    'struct msqid_ds'                 : 'msqid_ds',
    'struct necp_aggregate_result'    : 'necp_aggregate_result',
    'struct net_qos_param'            : 'net_qos_param',
    'struct ntptimeval'               : 'libc::ntptimeval',
    'struct nxctl_init'               : 'nxctl_init',
    'struct nxprov_reg'               : 'nxprov_reg',
    'struct pollfd'                   : 'libc::pollfd',
    'struct rlimit'                   : 'libc::rlimit',
    'struct rusage'                   : 'libc::rusage',
    'struct searchstate'              : 'searchstate',
    'struct sembuf'                   : 'libc::sembuf',
    'struct sf_hdtr'                  : 'libc::sf_hdtr',
    'struct shared_file_np'           : 'shared_file_np',
    'struct shmid_ds'                 : 'libc::shmid_ds',
    'struct sigaction'                : 'libc::sigaction',
    'struct sigaltstack'              : 'sigaltstack',
    'struct sigset_t'                 : 'libc::sigset_t',
    'struct sigvec'                   : 'sigvec',
    'struct sockaddr'                 : 'libc::sockaddr',
    'struct statfs'                   : 'libc::statfs',
    'struct statfs64'                 : 'libc::statfs',
    'struct timespec'                 : 'libc::timespec',
    'struct timeval'                  : 'libc::timeval',
    'struct timex'                    : 'libc::timex',
    'struct timezone'                 : 'libc::timezone',
    'struct ucontext'                 : 'libc::ucontext_t',
    'u_int'                           : 'u32',
    'u_int32_t'                       : 'u32',
    'u_long'                          : 'u64',
    'uid_t'                           : 'libc::uid_t',
    'uint32_t'                        : 'u32',
    'uint64_t'                        : 'u64',
    'uint8_t'                         : 'u8',
    'unsigned char'                   : 'u8',
    'unsigned int'                    : 'u32',
    'user_addr_t'                     : 'VMA',
    'user_size_t'                     : 'usize',
    'user_ssize_t'                    : 'isize',
    'uuid_t'                          : 'libc::uuid_t',
    'void'                            : '',
}

RUST_RESERVED = {
    'as',
    'async',
    'await',
    'break',
    'const',
    'continue',
    'crate',
    'dyn',
    'else',
    'enum',
    'extern',
    'false',
    'fn',
    'for',
    'if',
    'impl',
    'in',
    'let',
    'loop',
    'match',
    'mod',
    'move',
    'mut',
    'pub',
    'ref',
    'return',
    'self',
    'Self',
    'static',
    'struct',
    'super',
    'trait',
    'true',
    'type',
    'unsafe',
    'use',
    'where',
    'while',
}

POINTER_TYPES = {
    'Sz',
    'VMA',
}

STATUS_ONLY_SYSCALLS = {
    'sys_close',
    'link',
    'unlink',
    'sys_chdir',
    'sys_fchdir',
    'mknod',
    'chmod',
    'chown',
    'setuid',
    'getpeername',
    'getsockname',
    'access',
    'chflags',
    'fchflags',
    'sync',
    'kill',
    'sys_crossarch_trap',
    'sigaction',
    'sigprocmask',
    'getlogin',
    'setlogin',
    'acct',
    'sigpending',
    'sigaltstack',
    'ioctl',
    'reboot',
    'revoke',
    'symlink',
    'execve',
    'chroot',
    'msync',
    'oslog_coproc_reg',
    'oslog_coproc',
    'munmap',
    'mprotect',
    'madvise',
    'mincore',
    'setgroups',
    'setpgid',
    'setitimer',
    'swapon',
    'getitimer',
    'fsync',
    'setpriority',
    'connect',
    'bind',
    'setsockopt',
    'listen',
    'sigsuspend',
    'gettimeofday',
    'getrusage',
    'getsockopt',
    'settimeofday',
    'fchown',
    'fchmod',
    'setreuid',
    'setregid',
    'rename',
    'sys_flock',
    'mkfifo',
    'shutdown',
    'socketpair',
    'mkdir',
    'rmdir',
    'utimes',
    'futimes',
    'adjtime',
    'gethostuuid',
    'nfssvc',
    'statfs',
    'fstatfs',
    'unmount',
    'getfh',
    'funmount',
    'quotactl',
    'mount',
    'csops',
    'csops_audittoken',
    'waitid',
    'kdebug_typefilter',
    'kdebug_trace64',
    'kdebug_trace',
    'setgid',
    'setegid',
    'seteuid',
    'sigreturn',
    'sys_panic_with_data',
    'thread_selfcounts',
    'fdatasync',
    'stat',
    'sys_fstat',
    'lstat',
    'getrlimit',
    'setrlimit',
    'truncate',
    'ftruncate',
    'sysctl',
    'mlock',
    'munlock',
    'undelete',
    'getattrlist',
    'setattrlist',
    'getdirentriesattr',
    'exchangedata',
    'searchfs',
    'delete',
    'copyfile',
    'fgetattrlist',
    'fsetattrlist',
    'setxattr',
    'fsetxattr',
    'removexattr',
    'fremovexattr',
    'fsctl',
    'initgroups',
    'posix_spawn',
    'ffsctl',
    'minherit',
    'semsys',
    'msgsys',
    'shmsys',
    'semop',
    'msgctl',
    'msgsnd',
    'shmctl',
    'shmdt',
    'shm_unlink',
    'sem_close',
    'sem_unlink',
    'sem_wait',
    'sem_trywait',
    'sem_post',
    'sys_sysctlbyname',
    'umask_extended',
    'stat_extended',
    'lstat_extended',
    'sys_fstat_extended',
    'chmod_extended',
    'fchmod_extended',
    'access_extended',
    'sys_settid',
    'gettid',
    'setsgroups',
    'getsgroups',
    'setwgroups',
    'getwgroups',
    'mkfifo_extended',
    'mkdir_extended',
    'identitysvc',
    'shared_region_check_np',
    'psynch_rw_downgrade',
    'getsid',
    'sys_settid_with_pid',
    'psynch_cvclrprepost',
    'aio_fsync',
    'aio_suspend',
    'aio_read',
    'aio_write',
    'lio_listio',
    'iopolicysys',
    'process_policy',
    'mlockall',
    'munlockall',
    'issetugid',
    '__pthread_kill',
    '__pthread_sigmask',
    '__sigwait',
    '__disable_threadsignal',
    '__pthread_markcancel',
    '__pthread_canceled',
    '__semwait_signal',
    'sendfile',
    'stat64',
    'sys_fstat64',
    'lstat64',
    'stat64_extended',
    'lstat64_extended',
    'sys_fstat64_extended',
    'statfs64',
    'fstatfs64',
    '__pthread_chdir',
    '__pthread_fchdir',
    'audit',
    'auditon',
    'getauid',
    'setauid',
    'getaudit_addr',
    'setaudit_addr',
    'auditctl',
    'bsdthread_terminate',
    'lchown',
    'bsdthread_register',
    'workq_open',
    'ledger',
    '__mac_execve',
    '__mac_syscall',
    '__mac_get_file',
    '__mac_set_file',
    '__mac_get_link',
    '__mac_set_link',
    '__mac_get_proc',
    '__mac_set_proc',
    '__mac_get_fd',
    '__mac_set_fd',
    '__mac_get_pid',
    'sys_close_nocancel',
    'msync_nocancel',
    'fsync_nocancel',
    'connect_nocancel',
    'sigsuspend_nocancel',
    'waitid_nocancel',
    'msgsnd_nocancel',
    'sem_wait_nocancel',
    'aio_suspend_nocancel',
    '__sigwait_nocancel',
    '__semwait_signal_nocancel',
    '__mac_mount',
    '__mac_get_mount',
    '__mac_getfsstat',
    'audit_session_join',
    'sys_fileport_makeport',
    'audit_session_port',
    'pid_suspend',
    'pid_resume',
    'pid_hibernate',
    'pid_shutdown_sockets',
    'kas_info',
    'guarded_close_np',
    'change_fdguard_np',
    'usrctl',
    'proc_rlimit_control',
    'connectx',
    'disconnectx',
    'peeloff',
    'telemetry',
    'proc_uuid_policy',
    'memorystatus_get_level',
    'system_override',
    'vfs_purge',
    'sfi_ctl',
    'sfi_pidctl',
    'coalition',
    'coalition_info',
    'necp_match_policy',
    'clonefileat',
    'renameat',
    'faccessat',
    'fchmodat',
    'fchownat',
    'fstatat',
    'fstatat64',
    'linkat',
    'unlinkat',
    'symlinkat',
    'mkdirat',
    'getattrlistat',
    'proc_trace_log',
    'csrctl',
    'renameatx_np',
    'mremap_encrypted',
    'stack_snapshot_with_config',
    'microstackshot',
    'persona',
    'work_interval_ctl',
    'getentropy',
    '__nexus_register',
    '__nexus_deregister',
    '__nexus_create',
    '__nexus_destroy',
    '__nexus_get_opt',
    '__nexus_set_opt',
    '__channel_get_info',
    '__channel_sync',
    '__channel_get_opt',
    '__channel_set_opt',
    'fclonefileat',
    'fs_snapshot',
    'terminate_with_payload',
    'necp_session_action',
    'setattrlistat',
    'fmount',
    'ntp_gettime',
    'os_fault_with_payload',
    'coalition_ledger',
    'log_data',
    'objc_bp_assist_cfg_np',
    'shared_region_map_and_slide_2_np',
    'pivot_root',
    'task_inspect_for_pid',
    'task_read_for_pid',
    'tracker_action',
    'debug_syscall_reject',
    'sys_debug_syscall_reject_config',
    'graftdmg',
    'map_with_linking_np',
    'sys_record_system_event',
    'mkfifoat',
    'mknodat',
    'ungraftdmg',
    'sys_coalition_policy_set',
}

IMPLEMENTED_SYSCALLS = {}
SYSCALL_IMPL_FLAGS = {}

@dataclasses.dataclass
class Arg:
    name  : str
    type  : str
    indir : int

    @property
    def is_ptr(self) -> bool:
        return self.indir != 0 or self.type in POINTER_TYPES

    @property
    def rust_name(self) -> str:
        if self.name in RUST_RESERVED:
            return f'r#{self.name}'
        else:
            return self.name

    @property
    def rust_type(self) -> str:
        if self.indir != 0:
            return ' '.join(['*mut'] * self.indir + [self.type])
        elif self.type:
            return self.type
        else:
            return '()'

    def to_rust_args(self, i: int) -> tuple[int, str]:
        match self.rust_type:
            case 'u64'          : return 1, f'args[{i}]'
            case 'VMA'          : return 1, f'VMA::new(args[{i}])'
            case 'Sz'           : return 1, f'Sz::from(args[{i}] as *mut i8)'
            case 'semun_t'      : return 1, f'semun_t::from_u64(args[{i}])'
            case 'libc::uuid_t' : return 2, f'mk_uuid(args[{i}], args[{i + 1}])'
            case ty             : return 1, f'args[{i}] as {ty}'

@dataclasses.dataclass
class Syscall:
    id      : int
    name    : str
    args    : list[Arg]
    ret_ty  : Arg
    ret_int : bool

    @property
    def is_nosys(self) -> bool:
        return self.name in {'enosys', 'nosys'}

def to_snake_case(name: str):
    name = re.sub(r'[\-\.\s]', '_', name)
    return name[0].lower() + re.sub(r'[A-Z]', lambda m: '_' + m.group(0).lower(), name[1:])

with open('docs/syscalls.master') as fp:
    lines = fp.read().splitlines()

id = 0
state = 'normal'
syscalls = list[Syscall]()

for line in map(str.lstrip, lines):
    if not line or line.startswith(';') or line.startswith('#include'):
        continue

    if line.startswith('#if'):
        state = 'in_if'
        continue

    if line.startswith('#else'):
        assert state == 'in_if'
        state = 'in_else'
        continue

    if line.startswith('#endif'):
        assert state in ('in_if', 'in_else')
        state = 'normal'
        continue

    if state == 'in_else':
        continue

    number, _, _, decl = line.split(None, 3)
    number, decl = int(number), decl.strip()
    assert number == id, f'syscall decl mismatch: {line}'
    assert decl.startswith('{'), f'invalid syscall decl: {line}'

    end = decl.index('}')
    decl = decl[1:end].strip()
    ret_ty, decl = decl.split(None, 1)
    ret_int = ret_ty == 'int'
    ret_indir = 0

    while decl.startswith('*'):
        decl = decl[1:].strip()
        ret_ty += ' *'

    while ret_ty.endswith('*'):
        ret_ty = ret_ty[:-1].strip()
        ret_int = False
        ret_indir += 1

    ret_ty = TYPE_MAP[ret_ty]
    ret_ty = Arg('', ret_ty, ret_indir)

    func, decl = decl.split('(', 1)
    decl, *_ = decl.split(')')
    func = func.strip()
    args = []

    if decl.strip() != 'void':
        for arg in decl.split(','):
            arg = arg.strip()
            is_const = False
            *tys, name = arg.rsplit(None)

            if tys[0] == 'const':
                tys = tys[1:]
                is_const = True

            indir = 0
            ty = ' '.join(tys)

            while name.startswith('*'):
                name = name[1:].strip()
                ty += ' *'

            while ty.endswith('*'):
                ty = ty[:-1].strip()
                indir += 1

            if is_const and ty == 'char' and indir == 1:
                args.append(Arg(to_snake_case(name), 'Sz', 0))
                continue

            ty = TYPE_MAP[ty] or 'libc::c_void'
            args.append(Arg(to_snake_case(name), ty, indir))

    if len(args) > 8:
        raise RuntimeError(f'too many arguments: {line}')

    syscalls.append(Syscall(id, func, args, ret_ty, ret_int))
    id += 1

with open('src/aarch64/syscall/bsd.rs', 'w') as fp:
    print('//! Generated by `genbsdsyscalls.py`, DO NOT EDIT.', file = fp)
    print(file = fp)
    print(PRELUDE.strip(), file = fp)

    for sc in syscalls:
        if sc.args:
            print(file = fp)
            print('#[derive(Clone, Copy)]', file = fp)
            print(f'pub struct ARG_{sc.name} {{', file = fp)

            for arg in sc.args:
                print(f'    pub {arg.rust_name}: {arg.rust_type},', file = fp)

            print('}', file = fp)
            print(file = fp)
            print(f'impl Arg for ARG_{sc.name} {{', file = fp)
            print('    #[inline]', file = fp)
            print('    fn decode(args: &[u64; 9]) -> Self {', file = fp)
            print('        Self {', file = fp)
            argc = 0

            for arg in sc.args:
                n, value = arg.to_rust_args(argc)
                argc += n
                print(f'            {arg.rust_name}: {value},', file = fp)

            print('        }', file = fp)
            print('    }', file = fp)
            print('}', file = fp)
            print(file = fp)
            print(f'impl Debug for ARG_{sc.name} {{', file = fp)
            print('    #[inline]', file = fp)
            print("    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {", file = fp)
            print('        write!(f, "', end = '', file = fp)

            for i, arg in enumerate(sc.args):
                prefix = ', ' if i else ''
                print(f'{prefix}{arg.name}={{:?}}', end = '', file = fp)
            else:
                print('"', end = '', file = fp)

            for i, arg in enumerate(sc.args):
                print(f', self.{arg.rust_name}', end = '', file = fp)
            else:
                print(')', file = fp)

            print('    }', file = fp)
            print('}', file = fp)
            print(file = fp)

    print(file = fp)
    print('#[repr(u64)]', file = fp)
    print('#[derive(Debug, Clone, Copy)]', file = fp)
    print('pub enum BsdSyscall {', file = fp)

    for sc in syscalls:
        if not sc.is_nosys:
            if sc.args:
                print(f'    {sc.name}(ARG_{sc.name}) = {sc.id},', file = fp)
            else:
                print(f'    {sc.name} = {sc.id},', file = fp)

    print('    Unknown(u64)', file = fp)
    print('}', file = fp)
    print(file = fp)
    print('impl BsdSyscall {', file = fp)
    print('    pub fn decode(id: u64, args: &[u64; 9]) -> Self {', file = fp)
    print('        match id {', file = fp)

    for sc in syscalls:
        if not sc.is_nosys:
            if sc.args:
                print(f'            {sc.id} => Self::{sc.name}(Arg::decode(args)),', file = fp)
            else:
                print(f'            {sc.id} => Self::{sc.name},', file = fp)

    print('            _ => Self::Unknown(id),', file = fp)
    print('        }', file = fp)
    print('    }', file = fp)
    print('}', file = fp)
    print(file = fp)

with open('src/aarch64/syscall/bsd_impl.rs', 'w') as fp:
    print('//! Generated by `genbsdsyscalls.py`, DO NOT EDIT.', file = fp)
    print(file = fp)
    print(BSD_IMPL_PRELUDE.strip(), file = fp)

    for sc in syscalls:
        if not sc.is_nosys:
            if sc.ret_ty.is_ptr or any(v.is_ptr for v in sc.args) or sc.name in IMPLEMENTED_SYSCALLS:
                if sc.args:
                    print(f'            {sc.name}(args),', file = fp)
                else:
                    print(f'            {sc.name}(),', file = fp)
            else:
                if sc.args:
                    print(f'            {sc.name}(..),', file = fp)
                else:
                    print(f'            {sc.name},', file = fp)

    print('        }', file = fp)
    print('    }', file = fp)
    print('}', file = fp)

with open('src/aarch64/virtos/syscall/bsd/delegate.rs', 'w') as fp:
    print('//! Generated by `genbsdsyscalls.py`, DO NOT EDIT.', file = fp)
    print(file = fp)
    print('use std::io::Result as IoResult;', file = fp)
    print(file = fp)
    print('use crate::{', file = fp)
    print('    aarch64::{cpu::Cpu, syscall::bsd::*},', file = fp)
    print('    utils::ptr::VMA,', file = fp)
    print('};', file = fp)
    print(file = fp)

    for sc in syscalls:
        use_cpu = False
        use_args = False

        if sc.is_nosys:
            continue

        if not sc.ret_ty.is_ptr and all(not v.is_ptr for v in sc.args) and sc.name not in IMPLEMENTED_SYSCALLS:
            continue

        if flags := SYSCALL_IMPL_FLAGS.get(sc.name):
            use_cpu = 'use:cpu' in flags
            use_args = 'use:flags' in flags

        cpu = 'cpu' if use_cpu else '_cpu'
        ret_ty = sc.ret_ty.rust_type
        print('#[inline]', file = fp)

        if sc.name in STATUS_ONLY_SYSCALLS:
            ret_ty = '()'

        if sc.args:
            args = 'args' if use_args else '_args'
            print(f'pub fn {sc.name}({cpu}: &Cpu, {args}: ARG_{sc.name}) -> IoResult<{ret_ty}> {{', file = fp)
        else:
            print(f'pub fn {sc.name}({cpu}: &Cpu) -> IoResult<{ret_ty}> {{', file = fp)

        match IMPLEMENTED_SYSCALLS.get(sc.name):
            case None:
                print('    todo!();', file = fp)

            case ('code', code):
                print(code, file = fp)

            case pkg if isinstance(pkg, str):
                argv = []
                argv += ['cpu'] if use_cpu else []
                argv += ['args'] if use_args else []
                args = ', '.join(argv)
                print(f'    {pkg}::{sc.name}({args})', file = fp)

        print('}', file = fp)
        print(file = fp)

subprocess.check_call([
    'rustfmt',
    'src/aarch64/syscall/bsd.rs',
    'src/aarch64/syscall/bsd_impl.rs',
    'src/aarch64/virtos/syscall/bsd/delegate.rs',
])
