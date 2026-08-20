#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import json
import os
import subprocess
import sys

def usage():
    print(f'Usage: {sys.argv[0]} build [-r]')
    print(f'       {sys.argv[0]} run [-r] [--] args ...')
    sys.exit(1)

def build(is_debug: bool, run_after_build: bool, args: list[str]):
    def find_binary_name() -> str | None:
        for pkg in meta['packages']:
            for target in pkg['targets']:
                if target['kind'] == ['bin']:
                    return target['name']

    # get metadata
    meta = json.loads(subprocess.check_output([
        'cargo',
        'metadata',
        '--format-version',
        '1',
        '--no-deps',
    ]))

    # find binary name, the target path and workspace root
    profile = 'debug'
    build_cmd = ['cargo', 'build']
    binary_name = find_binary_name() or ''
    target_path = meta['target_directory']
    workspace_root = meta['workspace_root']

    # must have a binary name
    if not binary_name:
        raise FileNotFoundError('cannot find the output binary name')

    # select build flags and profile
    if not is_debug:
        profile = 'release'
        build_cmd.append('-r')

    # build the target
    subprocess.check_call(build_cmd, env = {
        **os.environ,
        '__BUILD_WITH_SIGN': 'yes',
    })

    # construct the args
    executable = f'{target_path}/{profile}/{binary_name}'
    args.insert(0, executable)

    # sign the target
    subprocess.check_call([
        '/usr/bin/codesign',
        '--sign',
        '-',
        '--entitlements',
        f'{workspace_root}/entitlements.xml',
        '--deep',
        '--force',
        executable,
    ])

    # run the binary if needed
    if run_after_build:
        os.execve(executable, args, os.environ)

match sys.argv[1:]:
    case ['run']                    : build(True, True, [])
    case ['run', '-r']              : build(False, True, [])
    case ['run', '--', *args]       : build(True, True, args)
    case ['run', '-r', '--', *args] : build(False, True, args)
    case ['build']                  : build(True, False, [])
    case ['build', '-r']            : build(False, False, [])
    case _                          : usage()
