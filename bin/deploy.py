#!/usr/bin/python

import sys
import json
import pathlib
import os.path
import os
import shutil

def normalize_path(path_string):
    return pathlib.Path(os.path.expanduser(os.path.expandvars(path_string))).absolute()

def evacuate(path):
    dest = path
    count = 0
    if path.is_symlink():
        path.unlink()
    while dest.exists():
        dest = path.parent / (path.name + f'.pkg_save_{count}')
        count += 1
    if dest != path:
        shutil.move(path, dest)

def traverse(f, pkg_path):
    json_path = pkg_path / 'pkg.json' 
    with open(pkg_path / 'pkg.json', 'r') as cfg_file:
        cfg = json.loads(cfg_file.read())

        # TODO
        if 'root' in cfg and cfg['root']:
            return

        acc = True

        if 'patch' in cfg:
            for patch in cfg['patch'].keys():
                source = normalize_path(pkg_path / patch)
                dest   = normalize_path(cfg['patch'][patch])
                acc &= f(cfg, source, dest)
                
        if 'fallback' in cfg:
            if cfg['fallback'] is not None:
                dest = normalize_path(cfg['fallback'])
                evacuate(dest)
                acc &= f(cfg, pkg_path, dest)

        return acc
    return False

def sync_file(cfg, source, dest):
    evacuate(dest)
    if cfg['link']:
        dest.symlink_to(source)
    else:
        shutil.copyfile(source, dest)

    return True

if __name__ == '__main__':
    cfg_path = normalize_path(sys.argv[1])
    with open(cfg_path, 'r') as f:
        cfg = json.loads(f.read())

        hooks_path = cfg_path.parent / cfg['hooks']
        pkgs_path = cfg_path.parent / cfg['pkgs']

        for pkg in pkgs_path.glob('*/'):
            traverse(sync_file, pkg)
