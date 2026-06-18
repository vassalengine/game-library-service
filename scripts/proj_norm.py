#!/usr/bin/python

import re
import sys
import unicodedata


def proj_norm(n):
    pat = re.compile(" +")

    n = pat.sub(' ', ''.join(
        c if unicodedata.category(c)[0] in ('L', 'N') else ' ' for c in unicodedata.normalize('NFKD', n).lower() if not unicodedata.category(c)[0] == 'M'
    )).strip()

    return n 


def run():
    print(proj_norm(sys.argv[1]))


if __name__ == '__main__':
    run()
