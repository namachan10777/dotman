#!/usr/bin/env python3

from genericpath import exists
import platform
import enum
import sys
import os
class OsName(enum.Enum):
    Unknown = 0
    MacOS = 1
    Linux = 2
    Windows = 3

def get_os():
    os_name = platform.system()
    os = OsName.Unknown
    if os_name == 'Linux':
        os = OsName.Linux
    elif os_name == 'MacOsName':
        os = OsName.MacOsName
    return os

def exit_with_err_msg(msg):
    print(msg, file=sys.stderr)
    exit(1)

def assert_with_msg(cond, msg):
    if not cond:
        exit_with_err_msg(msg)

class InstallUnit:
    def __init__(self, os_names) -> None:
        self.command_list = []
        self.os_names = os_names

    def execute_command_with_condition(self, command, condition):
        self.command_list.append((command, condition))

def execute_command_when_file_is_not_exists(command, path, os_names):
    unit = InstallUnit(os_names)
    unit.execute_command_with_condition(command, lambda: os.path.exists(path))

if __name__ == '__main__':
    print('== start system setup ==')
    os = get_os()
    assert_with_msg(os != OsName.Unknown, 'unknown os')
    units = [
        execute_command_when_file_is_not_exists([
            "/bin/bash", "-c",  "\"$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/master/install.sh)\""
        ], "/opt/brew/bin/brew", [OsName.MacOS])
    ]