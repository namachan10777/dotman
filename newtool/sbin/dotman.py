#!/usr/bin/env python3

import platform
import enum
import sys
class OS(enum.Enum):
    Unknown = 0
    MacOS = 1
    Linux = 2
    Windows = 3

def get_os():
    os_name = platform.system()
    os = OS.Unknown
    if os_name == 'Linux':
        os = OS.Linux
    elif os_name == 'MacOS':
        os = OS.MacOS
    return os

def exit_with_err_msg(msg):
    print(msg, file=sys.stderr)
    exit(1)

def assert_with_msg(cond, msg):
    if not cond:
        exit_with_err_msg(msg)

if __name__ == '__main__':
    print('== start system setup ==')
    os = get_os()
    assert_with_msg(os != OS.Unknown, 'unknown os')