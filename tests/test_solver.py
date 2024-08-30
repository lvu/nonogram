from unittest import TestCase

import numpy as np

from nonogram.solver import (
    line_to_str,
    new_field,
    fill_overlaps,
    only_way,
    solve_by_line,
    solve_line,
    str_to_line,
    verify_line,
)


class SolverTestCase(TestCase):
    def check_solve_line(self, hints, orig_str, result_str):
        line = str_to_line(orig_str)
        solve_line(hints, line)
        self.assertEqual(line_to_str(line), result_str)

    def test_verify_line(self):
        self.assertTrue(verify_line([1], str_to_line(".......")))
        self.assertTrue(verify_line([2, 3], str_to_line("......")))
        self.assertTrue(verify_line([2, 3], str_to_line("X..X.*.X")))
        self.assertTrue(verify_line([2, 3], str_to_line("X...X..*.X")))
        self.assertFalse(verify_line([2, 3], str_to_line("X..X*.X")))
        self.assertFalse(verify_line([2, 3], str_to_line("..*...")))
        self.assertFalse(verify_line([2, 3], str_to_line("X..*...X")))
        self.assertFalse(verify_line([2, 3], str_to_line("X..*...X")))
        self.assertFalse(verify_line([2, 3], str_to_line("*..*X...")))
        self.assertFalse(verify_line([2, 1], str_to_line("*..X.*.X*")))
        self.assertFalse(verify_line([2, 1], str_to_line(".*.X.*.X*")))
        self.assertFalse(verify_line([2, 1], str_to_line("XXXX.*.XX")))
        self.assertTrue(verify_line([1, 2], str_to_line("*..*X..")))

    def test_only_way(self):
        self.assertEqual(only_way([1, 2], str_to_line(".*.....*.X")), 2)
        self.assertEqual(only_way([1, 2, 3], str_to_line(".*..*.X")), 2)
        self.assertIsNone(only_way([1, 2], str_to_line("...*X")))

    def test_fill_overlaps(self):
        line = str_to_line("." * 10)
        fill_overlaps([3, 1, 2], line)
        self.assertEqual(line_to_str(line), "..*.......")

        line = str_to_line("." * 15)
        fill_overlaps([5, 2, 4], line)
        self.assertEqual(line_to_str(line), "..***......**..")

    def test_solve_line(self):
        self.check_solve_line([4], ".....*..", "XX..**..")
        self.check_solve_line([1, 2], "...*X..", ".X.*X..")
        self.check_solve_line([2, 1], ".X.X.*.X.", "XXXX.*.X*")
        self.check_solve_line([2, 1], "...X.*.X*", "XXXX.*.X*")

    def test_solve_by_line_full(self):
        row_hints = [[5], [1], [5], [1], [5]]
        col_hints = [[3, 1], [1, 1, 1], [1, 1, 1], [1, 1, 1], [1, 3]]
        field = new_field(len(row_hints), len(col_hints))
        solve_by_line(row_hints, col_hints, field)
        self.assertEqual([line_to_str(line) for line in field], [
            "*****",
            "*XXXX",
            "*****",
            "XXXX*",
            "*****",
        ])
