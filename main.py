import json
import sys
import traceback
from time import monotonic

from nonogram.katana_source import read_image
from nonogram.solver import line_to_str, new_field, solve_by_line, solve_by_line


if __name__ == '__main__':
    if len(sys.argv) > 1 and sys.argv[1] == "solve":
        data = json.load(sys.stdin)
        row_hints = data["row_hints"]
        col_hints = data["col_hints"]
        field = new_field(len(row_hints), len(col_hints))
        try:
            start = monotonic()
            solve_by_line(row_hints, col_hints, field)
            print(f"Elapsed: {(monotonic() - start) * 1000:.0f} ms")
        except Exception as err:
            tb = traceback.TracebackException.from_exception(err, capture_locals=True)
            print("".join(tb.format()))

        for line in field:
            print(line_to_str(line))

    elif len(sys.argv) > 1:
        print(json.dumps(read_image(sys.argv[1])))
    else:
        print(json.dumps(read_image(None)))
