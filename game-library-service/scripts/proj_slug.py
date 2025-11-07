#!/usr/bin/python

import re
import sys


def proj_slug(n):
    ws = re.compile('\\s')
    special = re.compile("[:\\/?#\\[\\]@!$&'()*+,;=%\"<>\\\\^`{}|]")
    ch = re.compile('-+')

    s = ws.sub('-', n)
    s = special.sub('', s)
    s = ch.sub('-', s)
    s = s.strip('-')
 
    return s   


def run():
    print(proj_slug(sys.argv[1]))


if __name__ == '__main__':
    run()
