#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import json
import os
import subprocess
import sys

from argparse import ArgumentParser
from argparse import Namespace

def cargo(command: str, *args: str) -> str:
    ps = subprocess.Popen(
        ['cargo', command, '--message-format=json-render-diagnostics', *args],
        stdout = subprocess.PIPE,
        env    = { **os.environ, '__BUILD_WITH_SIGN': 'yes' }
    )

    # get the entitlement path
    executable = None
    entitlement = os.path.join(os.path.abspath(os.path.dirname(__file__)), 'entitlements.xml')
    assert ps.stdout is not None

    # read json output line by line
    while ps.poll() is None:
        if line := ps.stdout.readline():
            if executable is None:
                data = json.loads(line)
                executable = data.get('executable')
        else:
            break

    # wait for cargo to end
    if ret := ps.wait():
        sys.exit(ret)

    # cargo must have produced an executable
    if executable is None:
        raise RuntimeError('cargo does not produce an executable')

    # construct the codesign command
    codesign_cmd = [
        '/usr/bin/codesign',
        '--sign',
        '-',
        '--entitlements',
        entitlement,
        '--deep',
        '--force',
        executable,
    ]

    # sign the target
    subprocess.check_call(codesign_cmd, executable = '/usr/bin/codesign')
    return executable

ps = ArgumentParser()
sub = ps.add_subparsers(required = True)

def do_build(args: Namespace):
    if args.release:
        cargo('build', '-r')
    else:
        cargo('build')

cmd = sub.add_parser('build', help = 'build tinyvm')
cmd.add_argument('-r', '--release', action = 'store_true', help = 'build for release')
cmd.set_defaults(handler = do_build)

def do_run(args: Namespace):
    target = cargo('build', '-r') if args.release else cargo('build')
    os.execve(target, [target] + args.args, os.environ)

cmd = sub.add_parser('run', help = 'run tinyvm')
cmd.add_argument('-r', '--release', action = 'store_true', help = 'run release build')
cmd.add_argument('args', nargs = '*')
cmd.set_defaults(handler = do_run)

def do_test(args: Namespace):
    target = cargo('test', '-p', 'tinyvm', '--no-run', '--', *args.args)
    os.execve(target, [target] + args.args, os.environ)

cmd = sub.add_parser('test', help = 'run tests for tinyvm')
cmd.add_argument('-r', '--release', action = 'store_true', help = 'run release build')
cmd.add_argument('args', nargs = '*')
cmd.set_defaults(handler = do_test)

args = ps.parse_args()
args.handler(args)