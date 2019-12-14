#!/usr/bin/python

import sys
import json

if __name__ == '__main__':
    with open(sys.argv[1], 'r') as f:
        cfg = json.loads(f.read())
        print(cfg)
