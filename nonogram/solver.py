from bisect import bisect_left
from itertools import groupby
from typing import Optional, Tuple

import numpy as np


FILLED = 1
EMPTY = -1
UNKNOWN = 0


VALUE_STR_MAP = {
    FILLED: "*",
    EMPTY: "X",
    UNKNOWN: ".",
}
STR_VALUIE_MAP = {v: k for k, v in VALUE_STR_MAP.items()}


def new_field(num_rows, num_cols) -> np.ndarray:
    return np.full((num_rows, num_cols), UNKNOWN, dtype=np.int8)


def get_consec_runs(line: np.ndarray) -> list[Tuple[int, int]]:
    nz, = np.nonzero(np.diff(line))
    splits = np.split(line, nz + 1)
    return [(s[0], s.shape[0]) for s in splits]


def get_last_filled(line: np.ndarray) -> int:
    return np.max(np.nonzero(line == FILLED), initial = -1)


def build_empty_maps(hints: tuple[int], line: np.ndarray) -> dict[int, np.ndarray]:
    empties = line == EMPTY
    return {
        hint: np.convolve(empties, np.ones(hint, dtype=bool), mode='valid')
        for hint in set(hints)
    }


def update_empty_maps(empty_maps: dict[int, np.ndarray], pos: int) -> dict[int, np.ndarray]:
    result = {hint: empty_map.copy() for hint, empty_map in empty_maps.items()}
    for hint, empty_map in result.items():
        empty_map[max(0, pos - hint + 1):pos + 1] = True
    return result


verify_line_cache = {}


def verify_line(
    hints: tuple[int], line: tuple[int],
    empty_maps: dict[int, np.ndarray], last_filled: int,
    offset: int = 0
) -> bool:
    cache_key = (hints, line[offset:])
    if (hit := verify_line_cache.get(cache_key)) is not None:
        return hit

    if not hints:
        result = offset > last_filled
        verify_line_cache[cache_key] = result
        return result

    current_hint = hints[0]
    empty_map = empty_maps[current_hint]
    size = len(line)
    if size < current_hint:
        verify_line_cache[cache_key] = False
        return False

    for start, val in enumerate(line[offset:size - current_hint + 1], offset):
        end = start + current_hint
        if (
            not empty_map[start]
            and (end == size or line[end] != FILLED)
            and verify_line(hints[1:], line, empty_maps, last_filled, end + 1)
        ):
            verify_line_cache[cache_key] = True
            return True
        if val == FILLED:
            verify_line_cache[cache_key] = False
            return False
    verify_line_cache[cache_key] = False
    return False


solve_line_cache = {}


def solve_line(hints: tuple[int], line: np.ndarray) -> None:
    """Solve what is possible in-place, return True if any changes were made."""

    cache_key = (hints, tuple(line))
    if (hit := solve_line_cache.get(cache_key)) is not None:
        if isinstance(hit, Exception):
            raise hit
        line[:] = hit
        return

    empty_maps = build_empty_maps(hints, line)
    last_filled: int = get_last_filled(line)
    if not verify_line(hints, tuple(line), empty_maps, last_filled):
        err = ValueError(f"Invalid line: {line_to_str(line)}; hints: {hints}")
        solve_line_cache[cache_key] = err
        raise err

    for idx, val in enumerate(line):
        if val == UNKNOWN:
            new_empty_maps = update_empty_maps(empty_maps, idx)
            line[idx] = FILLED
            if not verify_line(hints, tuple(line), empty_maps, max(last_filled, idx)):
                line[idx] = EMPTY
                empty_maps = new_empty_maps
                continue
            line[idx] = EMPTY
            if not verify_line(hints, tuple(line), new_empty_maps, last_filled):
                line[idx] = FILLED
                last_filled = max(last_filled, idx)
                continue
            line[idx] = UNKNOWN
    if verify_line(hints, tuple(line), empty_maps, last_filled):
        solve_line_cache[cache_key] = line.copy()
    else:
        err = ValueError(f"Solver resulted in invalid line: {line_to_str(line)}; hints: {hints}")
        solve_line_cache[cache_key] = err
        raise err


def solve_by_line(row_hints: list[tuple[int]], col_hints: list[tuple[int]], field: np.ndarray) -> None:
    """
    Try solving the nonogram inplace.

    Should be enough for "normal" nonograms.
    """
    num_rows, num_cols = field.shape
    assert len(row_hints) == num_rows
    assert len(col_hints) == num_cols

    for hints, line in zip(row_hints, field):
        solve_line(hints, line)

    changed_cols = range(num_cols)
    while True:
        changed_rows = set()
        for col_idx in changed_cols:
            line = field[:, col_idx]
            orig_line = line.copy()
            solve_line(col_hints[col_idx], line)
            changed_rows.update(np.nonzero(orig_line != line)[0])
        if not changed_rows:
            break

        changed_cols = set()
        for row_idx in changed_rows:
            line = field[row_idx]
            orig_line = line.copy()
            solve_line(row_hints[row_idx], line)
            changed_cols.update(np.nonzero(orig_line != line)[0])
        if not changed_cols:
            break


MAX_DEPTH = 2


def solve(row_hints: list[tuple[int]], col_hints: list[tuple[int]], field: np.ndarray, max_depth = MAX_DEPTH) -> list[np.ndarray]:
    solve_by_line(row_hints, col_hints, field)
    if np.all(field != UNKNOWN):
        return [field]
    result = []
    if max_depth == 0:
        return result
    for idxs, val in np.ndenumerate(field):
        if val != UNKNOWN:
            continue
        field_copy = field.copy()
        field_copy[idxs] = EMPTY
        try:
            result.extend(solve(row_hints, col_hints, field_copy, max_depth - 1))
        except ValueError:
            field[idxs] = FILLED
            continue
        field_copy = field.copy()
        field_copy[idxs] = FILLED
        try:
            result.extend(solve(row_hints, col_hints, field_copy, max_depth - 1))
        except ValueError:
            field[idxs] = EMPTY
            continue
    if max_depth == MAX_DEPTH:
        print("Not solved; found so far:")
        for line in field:
            print(line_to_str(line))
    result.sort(key=field_to_str)
    result = [next(grp) for _, grp in groupby(result, field_to_str)]
    return result


def line_to_str(line: np.ndarray) -> str:
    return ''.join(VALUE_STR_MAP[c] for c in line)


def field_to_str(field: np.ndarray) -> str:
    return "\n".join(map(line_to_str, field))


def str_to_line(s: str) -> np.ndarray:
    return np.asarray([STR_VALUIE_MAP[c] for c in s])
