from unittest import TestCase

import numpy as np
from numpy.testing import assert_array_equal

from nonogram.solver import (
    build_empty_maps,
    get_last_filled,
    line_to_str,
    new_field,
    solve_by_line,
    solve_line,
    str_to_line,
    verify_line,
)


def nomap_only_way(hints: list[int], line: np.ndarray):
    return only_way(tuple(hints), line, build_empty_maps(hints, line), get_last_filled(line))


def nomap_verify_line(hints: list[int], line: np.ndarray):
    return verify_line(tuple(hints), tuple(line), build_empty_maps(hints, line), get_last_filled(line))


class SolverTestCase(TestCase):
    def check_solve_line(self, hints, orig_str, result_str):
        line = str_to_line(orig_str)
        solve_line(tuple(hints), line)
        self.assertEqual(line_to_str(line), result_str)

    def test_verify_line(self):
        self.assertTrue(nomap_verify_line([1], str_to_line(".......")))
        self.assertTrue(nomap_verify_line([2, 3], str_to_line("......")))
        self.assertTrue(nomap_verify_line([2, 3], str_to_line("X..X.*.X")))
        self.assertTrue(nomap_verify_line([2, 3], str_to_line("X...X..*.X")))
        self.assertFalse(nomap_verify_line([2, 3], str_to_line("X..X*.X")))
        self.assertFalse(nomap_verify_line([2, 3], str_to_line("..*...")))
        self.assertFalse(nomap_verify_line([2, 3], str_to_line("X..*...X")))
        self.assertFalse(nomap_verify_line([2, 3], str_to_line("X..*...X")))
        self.assertFalse(nomap_verify_line([2, 3], str_to_line("*..*X...")))
        self.assertFalse(nomap_verify_line([2, 1], str_to_line("*..X.*.X*")))
        self.assertFalse(nomap_verify_line([2, 1], str_to_line(".*.X.*.X*")))
        self.assertFalse(nomap_verify_line([2, 1], str_to_line("XXXX.*.XX")))
        self.assertTrue(nomap_verify_line([1, 2], str_to_line("*..*X..")))

    def test_solve_line(self):
        self.check_solve_line([4], ".....*..", "XX..**..")
        self.check_solve_line([1, 2], "...*X..", ".X.*X..")
        self.check_solve_line([2, 1], ".X.X.*.X.", "XXXX.*.X*")
        self.check_solve_line([2, 1], "...X.*.X*", "XXXX.*.X*")

    def test_solve_by_line_full(self):
        row_hints = [tuple(line) for line in [[5], [1], [5], [1], [5]]]
        col_hints = [tuple(line) for line in [[3, 1], [1, 1, 1], [1, 1, 1], [1, 1, 1], [1, 3]]]
        field = new_field(len(row_hints), len(col_hints))
        solve_by_line(row_hints, col_hints, field)
        self.assertEqual([line_to_str(line) for line in field], [
            "*****",
            "*XXXX",
            "*****",
            "XXXX*",
            "*****",
        ])
