import contextlib
import sqlite3


def run():
    dbpath = 'projects.db'

    with contextlib.closing(sqlite3.connect(dbpath)) as conn:
        with conn as cur:
            for table in ('galleries', 'galleries_history'):
                rows = cur.execute(f"SELECT DISTINCT project_id FROM {table}")
                for r in rows:
                    gi = cur.execute(f"SELECT gallery_id, position FROM {table} WHERE project_id = ? ORDER BY position", r)
                    
                    gi = list(gi)

                    i = 1
                    while i < len(gi):
                        cur.execute(f"UPDATE {table} SET next_id = ? WHERE gallery_id = ?", (gi[i][0], gi[i-1][0]))
                        i = i + 1

                    i = len(gi) - 1
                    while i > 0:
                        cur.execute(f"UPDATE {table} SET prev_id = ? WHERE gallery_id = ?", (gi[i-1][0], gi[i][0]))
                        i = i - 1

                    print("")
                    gi = cur.execute(f"SELECT gallery_id, prev_id, next_id, position FROM {table} WHERE project_id = ? ORDER BY position", r)
                    for g in gi:
                        print(g)
                    

if __name__ == '__main__':
    run()
