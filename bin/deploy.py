#!/usr/bin/python

import sys
import json
import pathlib
import os.path
import os
import shutil

def normalize_path(path_string):
    return pathlib.Path(os.path.expanduser(os.path.expandvars(path_string))).resolve()

def install_pkg(pkg_path):
    with open(pkg_path / 'pkg.json', 'r') as f:
        cfg = json.loads(f.read())
        dest_cfg = cfg['dest']

        # TODO
        if 'root' in cfg and cfg['root']:
            return

        if 'fallback' in cfg:
            fallback = normalize_path(dest_cfg['fallback'])

        if 'patch' in dest_cfg:
            print(dest_cfg)
            for key in dest_cfg['patch'].keys():
                source = normalize_path(pkg_path / key)
                dest   = normalize_path(dest_cfg['patch'][key])
                print(source, dest)
                if cfg['link']:
                    os.symlink(source, dest)
                else:
                    shutil.copyfile(source, dest)
            


if __name__ == '__main__':
    cfg_path = normalize_path(sys.argv[1])
    with open(cfg_path, 'r') as f:
        cfg = json.loads(f.read())

        hooks_path = cfg_path.parent / cfg['hooks']
        pkgs_path = cfg_path.parent / cfg['pkgs']

        for pkg in pkgs_path.glob('*/'):
            install_pkg(pkg)
