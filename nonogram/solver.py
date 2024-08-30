from bisect import bisect_left
from itertools import groupby
from typing import Optional, Tuple

import numpy as np
from line_profiler import profile


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


def consec_runs(line: np.ndarray) -> list[Tuple[int, int]]:
    nz, = np.nonzero(np.diff(line))
    splits = np.split(line, nz + 1)
    return [(s[0], s.shape[0]) for s in splits]


def empty_intervals(line: np.ndarray) -> list[Tuple[int, int]]:
    padded = np.zeros((line.shape[0] + 2,), dtype=line.dtype)
    padded[1:-1] = line
    diff = np.diff((padded == EMPTY) * 1)
    starts, = np.nonzero(diff == 1)
    ends, = np.nonzero(diff == -1)
    return [(start, end) for start, end in zip(starts, ends)]


def update_empty_intervals(empty: list[Tuple[int, int]], pos: int) -> None:
    if not empty or pos < empty[0][0] - 1:
        empty.insert(0, (pos, pos + 1))
        return

    starts = [start for start, _ in empty]
    idx = bisect_left(starts, pos)
    if idx > 0 and idx < len(empty) and empty[idx - 1][1] == pos == empty[idx][0] - 1:
        empty[idx - 1] = (empty[idx - 1][0], empty[idx][1])
        del empty[idx]
    elif idx > 0 and empty[idx - 1][1] == pos:
        empty[idx - 1] = (empty[idx - 1][0], pos + 1)
    elif idx < len(empty) and pos == empty[idx][0] - 1:
        empty[idx] = (pos, empty[idx][1])
    else:
        empty.insert(idx, (pos, pos + 1))


@profile
def intersects_empty(empty: list[Tuple[int, int]], start: int, end: int) -> bool:
    starts = [s for s, _ in empty]
    idx = bisect_left(starts, start)
    return idx < len(empty) and end > empty[idx][0]


@profile
def verify_line(
    hints: list[int], line: np.ndarray,
    empty: Optional[list[Tuple[int, int]]] = None, empty_offset: int = 0
) -> bool:
    if not hints:
        return bool(np.all(line != FILLED))

    if empty is None:
        empty = empty_intervals(line)

    current_hint = hints[0]
    size = line.shape[0]
    if size < current_hint:
        return False

    for start, val in enumerate(line[:size - current_hint + 1]):
        end = start + current_hint
        if (
            val != EMPTY
            and not intersects_empty(empty, empty_offset + start, empty_offset + end)
            and (end == size or line[end] != FILLED)
            and verify_line(hints[1:], line[end + 1:], empty, empty_offset + end + 1)
        ):
            return True
        if val == FILLED:
            return False
    return False


def only_way(hints: list[int], line: np.ndarray) -> Optional[int]:
    one_way = False
    for n_hints in range(1, len(hints) + 1):
        if verify_line(hints[:n_hints], line):
            if not one_way:
                one_way = True
            else:
                return None
        elif one_way:
            return n_hints - 1
    assert one_way
    return len(hints)


def fill_overlaps(hints: list[int], line: np.ndarray) -> None:
    orig_line = line.copy()
    mask = np.full((hints[0],), 1)
    for hint_idx, hint in enumerate(hints[1:], 2):
        mask = np.append(mask, 0)
        mask = np.concatenate((mask, np.full((hint,), hint_idx)))
    left = np.pad(mask, (0, line.shape[0] - mask.shape[0]), constant_values=0)
    right = np.pad(mask, (line.shape[0] - mask.shape[0], 0), constant_values=0)
    overlap = (left == right) & (right != 0)
    if np.any(line[overlap] == EMPTY):
        raise RuntimeError(f"Invalid overlap for line {line_to_str(line)}; hints: {hints}")
    line[overlap] = FILLED
    if not verify_line(hints, line):
        raise ValueError(f"Overlap resulted in invalid line: {line_to_str(line)}; src: {line_to_str(orig_line)}, hints: {hints}")


@profile
def solve_line(hints: list[int], line: np.ndarray) -> None:
    """Solve what is possible in-place, return True if any changes were made."""
    if not verify_line(hints, line):
        raise ValueError(f"Invalid line: {line_to_str(line)}; hints: {hints}")

    nz, = np.nonzero(line != EMPTY)
    if len(nz) > 1:
        line = line[:nz[-1] + 1]
    if len(nz) > 0:
        line = line[nz[0]:]

    nz, = np.nonzero(np.diff((line == EMPTY) * 1) == 1)
    parts = [p for p in np.split(line, nz + 1) if np.any(p != EMPTY)]
    if len(parts) > 1:
        if np.any(parts[0] == FILLED) and (num_hints := only_way(hints, parts[0])) is not None:
            solve_line(hints[:num_hints], parts[0])
            solve_line(hints[num_hints:], line[parts[0].shape[0]:])
            return
        if np.any(parts[-1] == FILLED) and (num_hints := only_way(hints[::-1], parts[-1][::-1])) is not None:
            solve_line(hints[-num_hints:], parts[-1])
            solve_line(hints[:-num_hints], line[:sum(p.shape[0] for p in parts[:-1])])
            return

    if np.sum(line == FILLED) == sum(hints):
        line[line == UNKNOWN] = EMPTY
        return

    if np.all(line == UNKNOWN) and sum(hints) + len(hints) + max(hints) - 1 < line.shape[0]:
        return

    empty = empty_intervals(line)
    for idx, val in enumerate(line):
        if val == UNKNOWN:
            line[idx] = FILLED
            if not verify_line(hints, line, empty):
                line[idx] = EMPTY
                update_empty_intervals(empty, idx)
                continue
            line[idx] = EMPTY
            empty_copy = empty[:]
            update_empty_intervals(empty_copy, idx)
            if not verify_line(hints, line, empty_copy):
                line[idx] = FILLED
                continue
            line[idx] = UNKNOWN
    if not verify_line(hints, line, empty):
        raise ValueError(f"Solver resulted in invalid line: {line_to_str(line)}; hints: {hints}")


@profile
def solve_by_line(row_hints: list[list[int]], col_hints: list[list[int]], field: np.ndarray) -> None:
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


def line_to_str(line: np.ndarray) -> str:
    return ''.join(VALUE_STR_MAP[c] for c in line)


def str_to_line(s: str) -> np.ndarray:
    return np.asarray([STR_VALUIE_MAP[c] for c in s])
