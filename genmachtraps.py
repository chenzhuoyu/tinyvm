#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import dataclasses
import subprocess

PRELUDE = r'''#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::fmt::{Debug, Formatter, Result as FmtResult};

use crate::{macros::define_bit_field, utils::ptr::Uintptr};

pub type mach_msg_priority_t = u32;
pub type mach_port_flavor_t = i32;
pub type mach_port_info_t = *mut i32;
pub type mach_port_name_array_t = *mut mach2::port::mach_port_name_t;

pub const MACH_MSG_OPTION_NONE: u64 = 0x00000000;
pub const MACH_SEND_MSG: u64 = 0x00000001;
pub const MACH_RCV_MSG: u64 = 0x00000002;
pub const MACH_RCV_LARGE: u64 = 0x00000004;
pub const MACH_RCV_LARGE_IDENTITY: u64 = 0x00000008;
pub const MACH_SEND_TIMEOUT: u64 = 0x00000010;
pub const MACH_SEND_OVERRIDE: u64 = 0x00000020;
pub const MACH_SEND_INTERRUPT: u64 = 0x00000040;
pub const MACH_SEND_NOTIFY: u64 = 0x00000080;
pub const MACH_SEND_ALWAYS: u64 = 0x00010000;
pub const MACH_SEND_FILTER_NONFATAL: u64 = 0x00010000;
pub const MACH_SEND_TRAILER: u64 = 0x00020000;
pub const MACH_SEND_NOIMPORTANCE: u64 = 0x00040000;
pub const MACH_SEND_NODENAP: u64 = MACH_SEND_NOIMPORTANCE;
pub const MACH_SEND_IMPORTANCE: u64 = 0x00080000;
pub const MACH_SEND_SYNC_OVERRIDE: u64 = 0x00100000;
pub const MACH_SEND_PROPAGATE_QOS: u64 = 0x00200000;
pub const MACH_SEND_SYNC_USE_THRPRI: u64 = MACH_SEND_PROPAGATE_QOS;
pub const MACH_SEND_KERNEL: u64 = 0x00400000;
pub const MACH_SEND_SYNC_BOOTSTRAP_CHECKIN: u64 = 0x00800000;
pub const MACH_RCV_TIMEOUT: u64 = 0x00000100;
pub const MACH_RCV_NOTIFY: u64 = 0x00000000;
pub const MACH_RCV_INTERRUPT: u64 = 0x00000400;
pub const MACH_RCV_VOUCHER: u64 = 0x00000800;
pub const MACH_RCV_OVERWRITE: u64 = 0x00000000;
pub const MACH_RCV_GUARDED_DESC: u64 = 0x00001000;
pub const MACH_RCV_SYNC_WAIT: u64 = 0x00004000;
pub const MACH_RCV_SYNC_PEEK: u64 = 0x00008000;
pub const MACH_MSG_STRICT_REPLY: u64 = 0x00000200;

bitflags::bitflags! {
    #[repr(C)]
    #[derive(Debug, Clone, Copy)]
    pub struct mach_msg_option64_t : u64 {
        const MACH64_MSG_OPTION_NONE             = 0;
        const MACH64_SEND_MSG                    = MACH_SEND_MSG;
        const MACH64_RCV_MSG                     = MACH_RCV_MSG;
        const MACH64_RCV_LARGE                   = MACH_RCV_LARGE;
        const MACH64_RCV_LARGE_IDENTITY          = MACH_RCV_LARGE_IDENTITY;
        const MACH64_SEND_TIMEOUT                = MACH_SEND_TIMEOUT;
        const MACH64_SEND_OVERRIDE               = MACH_SEND_OVERRIDE;
        const MACH64_SEND_INTERRUPT              = MACH_SEND_INTERRUPT;
        const MACH64_SEND_NOTIFY                 = MACH_SEND_NOTIFY;
        const MACH64_SEND_ALWAYS                 = MACH_SEND_ALWAYS;
        const MACH64_SEND_IMPORTANCE             = MACH_SEND_IMPORTANCE;
        const MACH64_SEND_KERNEL                 = MACH_SEND_KERNEL;
        const MACH64_SEND_FILTER_NONFATAL        = MACH_SEND_FILTER_NONFATAL;
        const MACH64_SEND_TRAILER                = MACH_SEND_TRAILER;
        const MACH64_SEND_NOIMPORTANCE           = MACH_SEND_NOIMPORTANCE;
        const MACH64_SEND_NODENAP                = MACH_SEND_NODENAP;
        const MACH64_SEND_SYNC_OVERRIDE          = MACH_SEND_SYNC_OVERRIDE;
        const MACH64_SEND_PROPAGATE_QOS          = MACH_SEND_PROPAGATE_QOS;
        const MACH64_SEND_SYNC_BOOTSTRAP_CHECKIN = MACH_SEND_SYNC_BOOTSTRAP_CHECKIN;
        const MACH64_RCV_TIMEOUT                 = MACH_RCV_TIMEOUT;
        const MACH64_RCV_INTERRUPT               = MACH_RCV_INTERRUPT;
        const MACH64_RCV_VOUCHER                 = MACH_RCV_VOUCHER;
        const MACH64_RCV_GUARDED_DESC            = MACH_RCV_GUARDED_DESC;
        const MACH64_RCV_SYNC_WAIT               = MACH_RCV_SYNC_WAIT;
        const MACH64_RCV_SYNC_PEEK               = MACH_RCV_SYNC_PEEK;
        const MACH64_MSG_STRICT_REPLY            = MACH_MSG_STRICT_REPLY;
        const MACH64_MSG_VECTOR                  = 0x0000000100000000;
        const MACH64_SEND_KOBJECT_CALL           = 0x0000000200000000;
        const MACH64_SEND_MQ_CALL                = 0x0000000400000000;
        const MACH64_SEND_ANY                    = 0x0000000800000000;
        const MACH64_SEND_DK_CALL                = 0x0000001000000000;
        const MACH64_POLICY_KERNEL_EXTENSION     = 0x0000002000000000;
        const MACH64_POLICY_FILTER_NON_FATAL     = 0x0000004000000000;
        const MACH64_POLICY_FILTER_MSG           = 0x0000008000000000;
        const MACH64_POLICY_DEFAULT              = 0x0000010000000000;
        const MACH64_POLICY_ENHANCED             = 0x0000020000000000;
        const MACH64_POLICY_PLATFORM             = 0x0000040000000000;
        const MACH64_POLICY_KERNEL               = 0x0000100000000000;
        const MACH64_POLICY_SIMULATED            = 0x0000200000000000;
        const MACH64_POLICY_TRANSLATED           = 0x0000400000000000;
        const MACH64_POLICY_OPTED_OUT            = 0x0000800000000000;
        const MACH64_POLICY_ENHANCED_V0          = 0x0001000000000000;
        const MACH64_POLICY_ENHANCED_V1          = 0x0002000000000000;
        const MACH64_POLICY_ENHANCED_V2          = 0x0004000000000000;
        const MACH64_RCV_LINEAR_VECTOR           = 0x1000000000000000;
        const MACH64_RCV_STACK                   = 0x2000000000000000;
        const MACH64_MACH_MSG2                   = 0x8000000000000000;
    }
}

impl mach_msg_option64_t {
    pub const MACH64_POLICY_ENHANCED_VERSION_MASK: Self = {
        Self::MACH64_POLICY_ENHANCED_V0
            .union(Self::MACH64_POLICY_ENHANCED_V1)
            .union(Self::MACH64_POLICY_ENHANCED_V2)
    };

    pub const MACH64_POLICY_MASK: Self = {
        Self::MACH64_POLICY_DEFAULT
            .union(Self::MACH64_POLICY_ENHANCED)
            .union(Self::MACH64_POLICY_PLATFORM)
            .union(Self::MACH64_POLICY_KERNEL)
            .union(Self::MACH64_POLICY_SIMULATED)
            .union(Self::MACH64_POLICY_TRANSLATED)
            .union(Self::MACH64_POLICY_OPTED_OUT)
    };
}

define_bit_field! {
    pub struct mach_msg_packed32_t : u64 {
        lsb: 32,
        msb: 32,
    }
}

trait Arg {
    fn decode(args: &[u64; 9]) -> Self;
}
'''

TYPE_MAP = {
    'boolean_t'                            : 'i32',
    'clock_res_t'                          : 'mach2::clock_types::clock_res_t',
    'int'                                  : 'i32',
    'ipc_space_t'                          : 'mach2::mach_types::ipc_space_t',
    'kern_return_t'                        : 'mach2::kern_return_t',
    'mach_error_t'                         : 'mach2::mach_error_t',
    'mach_msg_header_t'                    : 'mach2::message::mach_msg_header_t',
    'mach_msg_id_t'                        : 'mach2::message::mach_msg_id_t',
    'mach_msg_option_t'                    : 'mach2::message::mach_msg_option_t',
    'mach_msg_option64_t'                  : 'mach_msg_option64_t',
    'mach_msg_packed32_t'                  : 'mach_msg_packed32_t',
    'mach_msg_priority_t'                  : 'mach_msg_priority_t',
    'mach_msg_return_t'                    : 'mach2::message::mach_msg_return_t',
    'mach_msg_size_t'                      : 'mach2::message::mach_msg_size_t',
    'mach_msg_timeout_t'                   : 'mach2::message::mach_msg_timeout_t',
    'mach_msg_type_name_t'                 : 'mach2::message::mach_msg_type_name_t',
    'mach_msg_type_number_t'               : 'mach2::message::mach_msg_type_number_t',
    'mach_port_delta_t'                    : 'mach2::port::mach_port_delta_t',
    'mach_port_flavor_t'                   : 'mach_port_flavor_t',
    'mach_port_info_t'                     : 'mach_port_info_t',
    'mach_port_mscount_t'                  : 'mach2::port::mach_port_mscount_t',
    'mach_port_name_array_t'               : 'mach_port_name_array_t',
    'mach_port_name_t'                     : 'mach2::port::mach_port_name_t',
    'mach_port_options_t'                  : 'mach2::port::mach_port_options_t',
    'mach_port_right_t'                    : 'mach2::port::mach_port_right_t',
    'mach_port_t'                          : 'mach2::port::mach_port_t',
    'mach_port_type_t'                     : 'mach2::port::mach_port_type_t',
    'mach_timespec_t'                      : 'mach2::clock_types::mach_timespec_t',
    'mach_vm_address_t'                    : 'Uintptr',
    'mach_vm_offset_t'                     : 'mach2::vm_types::mach_vm_offset_t',
    'mach_vm_size_t'                       : 'mach2::vm_types::mach_vm_size_t',
    'mach_voucher_attr_key_t'              : 'mach2::mach_voucher_types::mach_voucher_attr_key_t',
    'mach_voucher_attr_raw_recipe_array_t' : 'mach2::mach_voucher_types::mach_voucher_attr_raw_recipe_array_t',
    'mach_voucher_attr_raw_recipe_t'       : 'mach2::mach_voucher_types::mach_voucher_attr_raw_recipe_t',
    'natural_t'                            : 'u32',
    'sleep_type_t'                         : 'mach2::clock_types::sleep_type_t',
    'uint64_t'                             : 'u64',
    'unsigned int'                         : 'u32',
    'vm_prot_t'                            : 'mach2::vm_prot::vm_prot_t',
    'vm_purgable_t'                        : 'mach2::vm_purgable::vm_purgable_t',
    'void'                                 : 'libc::c_void',
}

SHOW_AS_HEX = {
    'mach2::mach_types::ipc_space_t',
    'mach2::message::mach_msg_option_t',
    'mach2::port::mach_port_name_t',
    'mach2::port::mach_port_t',
    'mach2::port::mach_port_type_t',
    'mach2::vm_prot::vm_prot_t',
    'mach2::vm_types::mach_vm_offset_t',
    'mach2::vm_types::mach_vm_size_t',
    'u64',
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

@dataclasses.dataclass
class Arg:
    name  : str
    type  : str
    indir : int

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
        else:
            return self.type

    def to_rust_args(self, i: int) -> tuple[int, str]:
        match self.rust_type:
            case 'u64'                 : return 1, f'args[{i}]'
            case 'Uintptr'             : return 1, f'Uintptr::from(args[{i}])'
            case 'mach_msg_option64_t' : return 1, f'mach_msg_option64_t::from_bits_retain(args[{i}])'
            case 'mach_msg_packed32_t' : return 1, f'mach_msg_packed32_t(args[{i}])'
            case ty                    : return 1, f'args[{i}] as {ty}'

@dataclasses.dataclass
class MachTrap:
    id: int
    name: str
    argc: int
    wordc: int
    fn_sig: list[Arg]
    ret_ty: Arg
    ret_port: bool

SPECIAL = {
    'pfz_exit': {
        'ret_ty': Arg('', 'kern_return_t', 0),
    }
}

with open('docs/syscall_sw.c') as fp:
    lines = fp.read().splitlines()

state = ['discard']
trap_table = list[MachTrap]()

for line in lines:
    line = line.strip()
    parts = line.split()

    if parts[:3] == ['const', 'mach_trap_t', 'mach_trap_table[MACH_TRAP_TABLE_COUNT]']:
        assert state[-1] == 'discard'
        state[-1] = 'trap_table'
        continue

    if state[-1] == 'trap_table' and parts[:1] == ['};']:
        state[-1] = 'discard'
        continue

    if state[-1] == 'discard':
        continue

    if line.startswith('#if'):
        state.append('in_if')
        continue

    if line.startswith('#else'):
        assert state[-1] == 'in_if'
        state[-1] = 'in_else'
        continue

    if line.startswith('#endif'):
        assert state.pop() in {'in_if', 'in_else'}
        continue

    if state[-1] == 'in_else':
        continue

    try:
        _, num, _, decl = line.split(None, 3)
        num = int(num)
    except ValueError:
        continue

    assert num == len(trap_table), 'misaligned trap entry'
    args = decl[decl.index('(') + 1:decl.index(')')].split(',')
    name, argc, wordc, munge, *extras = [arg.strip() for arg in args]
    argc, wordc, ret_port = int(argc), int(wordc), False

    for extra in extras:
        match extra.split():
            case ['.mach_trap_returns_port', '=', retc]:
                assert retc == '1', 'invalid mach trap return port'
                ret_port = True
                break

    ret_ty = Arg('', '', 0)
    trap_table.append(MachTrap(num, name, argc, wordc, [], ret_ty, ret_port))
    num += 1

with open('docs/mach_traps.h') as fp:
    lines = fp.read().splitlines()

state = []
sources = []
trap_map = dict[str, MachTrap]()

for trap in trap_table:
    if trap.name != 'kern_invalid':
        trap_map[trap.name] = trap

for name, patch in SPECIAL.items():
    trap_map[name].__dict__.update(patch)

for line in lines:
    line = line.strip()
    parts = line.split()

    if line == '__END_DECLS':
        break

    if line == '__BEGIN_DECLS':
        assert not state
        state.append('normal')
        continue

    if not state or not parts:
        continue

    match parts[0]:
        case '#if' | '#ifdef' | '#ifndef':
            state.append('in_if')
            continue

        case '#else':
            assert state[-1] == 'in_if'
            state[-1] = 'in_else'
            continue

        case '#endif':
            assert state.pop() in {'in_if', 'in_else'}
            continue

    if 'in_else' not in state:
        sources.append(line)

source = ' '.join(sources)
source = '\n'.join(s.strip() for s in source.split(';'))
source = subprocess.check_output(['/usr/bin/cpp', '-E'], input = source, text = True)

for line in source.splitlines():
    line = line.strip()
    parts = line.split(None, 2)

    if not parts or parts[0] == '#':
        continue

    ret_indir = 0
    extern, ret_ty, decl = parts
    assert extern == 'extern'

    while decl.startswith('*'):
        decl = decl[1:].strip()
        ret_ty += ' *'

    while ret_ty.endswith('*'):
        ret_ty = ret_ty[:-1].strip()
        ret_indir += 1

    name, _, decl = decl.partition('(')
    args = decl[:decl.index(')')].split(',')
    name = name.strip()

    if name not in trap_map:
        name += '_trap'

    trap = trap_map[name]
    trap.ret_ty = Arg('', TYPE_MAP[ret_ty], ret_indir)

    if args == ['void']:
        continue

    for arg in args:
        arg = arg.split()
        *arg_ty, arg_name = arg
        arg_indir = 0
        arg_ty = ' '.join(arg_ty)

        while arg_name.startswith('*'):
            arg_name = arg_name[1:].strip()
            arg_ty += ' *'

        while arg_ty.endswith('*'):
            arg_ty = arg_ty[:-1].strip()
            arg_indir += 1

        arg = Arg(arg_name, TYPE_MAP[arg_ty], arg_indir)
        trap.fn_sig.append(arg)

with open('src/aarch64/syscall/mach.rs', 'w') as fp:
    print('//! Generated by `genmachtraps.py`, DO NOT EDIT.', file = fp)
    print(file = fp)
    print(PRELUDE.strip(), file = fp)

    for trap in trap_table:
        if trap.fn_sig:
            print(file = fp)
            print('#[derive(Clone, Copy)]', file = fp)
            print(f'pub struct ARG_{trap.name} {{', file = fp)

            for arg in trap.fn_sig:
                print(f'    pub {arg.rust_name}: {arg.rust_type},', file = fp)

            print('}', file = fp)
            print(file = fp)
            print(f'impl Arg for ARG_{trap.name} {{', file = fp)
            print('    #[inline]', file = fp)
            print('    fn decode(args: &[u64; 9]) -> Self {', file = fp)
            print('        Self {', file = fp)
            argc = 0

            for arg in trap.fn_sig:
                n, value = arg.to_rust_args(argc)
                argc += n
                print(f'            {arg.rust_name}: {value},', file = fp)

            print('        }', file = fp)
            print('    }', file = fp)
            print('}', file = fp)
            print(file = fp)
            print(f'impl Debug for ARG_{trap.name} {{', file = fp)
            print('    #[inline]', file = fp)
            print("    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {", file = fp)
            print('        write!(f, "', end = '', file = fp)

            for i, arg in enumerate(trap.fn_sig):
                prefix = ', ' if i else ''
                fmtspec = '0x{:x}' if arg.rust_type in SHOW_AS_HEX else '{:?}'
                print(f'{prefix}{arg.name}={fmtspec}', end = '', file = fp)
            else:
                print('"', end = '', file = fp)

            for i, arg in enumerate(trap.fn_sig):
                print(f', self.{arg.rust_name}', end = '', file = fp)
            else:
                print(')', file = fp)

            print('    }', file = fp)
            print('}', file = fp)
            print(file = fp)

    print(file = fp)
    print('#[repr(u64)]', file = fp)
    print('#[derive(Debug, Clone, Copy)]', file = fp)
    print('pub enum MachTrap {', file = fp)

    for trap in trap_table:
        if trap.name != 'kern_invalid':
            if trap.fn_sig:
                print(f'    {trap.name}(ARG_{trap.name}) = {trap.id},', file = fp)
            else:
                print(f'    {trap.name} = {trap.id},', file = fp)

    print('    Unknown(u64)', file = fp)
    print('}', file = fp)
    print(file = fp)
    print('impl MachTrap {', file = fp)
    print('    pub fn decode(id: u64, args: &[u64; 9]) -> Self {', file = fp)
    print('        match id {', file = fp)

    for trap in trap_table:
        if trap.name != 'kern_invalid':
            if trap.fn_sig:
                print(f'            {trap.id} => Self::{trap.name}(Arg::decode(args)),', file = fp)
            else:
                print(f'            {trap.id} => Self::{trap.name},', file = fp)

    print('            _ => Self::Unknown(id),', file = fp)
    print('        }', file = fp)
    print('    }', file = fp)
    print('}', file = fp)
    print(file = fp)

subprocess.check_call([
    'rustfmt',
    'src/aarch64/syscall/mach.rs',
])
