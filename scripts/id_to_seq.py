#!/usr/bin/python

import contextlib
import sqlite3


def run():
    dbpath = 'projects.db'
   
    with contextlib.closing(sqlite3.connect(dbpath)) as conn:
        with conn as cur:
            rows = cur.execute('SELECT DISTINCT project_id FROM galleries_history ORDER BY project_id')
            for r in rows:
                glist = []
                grows = cur.execute('SELECT gallery_id, next_id FROM galleries_history WHERE project_id = ? ORDER BY prev_id NULLS FIRST', r)

                x = list(grows)

                glist.append(x[0])

                m = { g[0]: i for i, g in enumerate(x) }

                while glist[-1][1] != None:
                    glist.append(x[m[glist[-1][1]]])

                slots = len(glist) + 1

                sk = [(bytes([round((i + 1) / slots * 255)]), g[0]) for i, g in enumerate(glist)]

                for s in sk:
                    cur.execute('UPDATE galleries_history SET sort_key = ? WHERE gallery_id = ?', s)


if __name__ == '__main__':
    run()
