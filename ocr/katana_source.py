import os.path
from enum import auto, Enum
from itertools import groupby
from string import digits
from typing import Iterable, NamedTuple, Optional, TypeVar, Tuple

import cv2
import numpy as np
from PIL import Image, ImageGrab


RED = (0, 0, 0xFF)
BLUE = (0, 0xFF, 0)

T = TypeVar("T")


class Rect(NamedTuple):
    x: int
    y: int
    w: int
    h: int


class Direction(Enum):
    rows = auto()
    cols = auto()


class DigitOCR:

    def __init__(self, sample_path: str):
        self.digits_img = cv2.cvtColor(cv2.imread(sample_path), cv2.COLOR_BGR2GRAY)
        h, w, *_ = self.digits_img.shape
        self.char_height = h

        self.x_broders = []
        rects = get_contour_rects(self.digits_img)
        assert len(rects) == 10
        rects.sort(key=lambda r: r.x)
        for r1, r2 in pairwise(rects):
            self.x_broders.append((r1.x + r1.w - 1 + r2.x) // 2)
        self.x_broders.append(w - 1)

    def recognize(self, img: np.ndarray) -> int:
        h, w, *_ = img.shape
        new_w = self.char_height * w  // h
        img = cv2.resize(img, (new_w, self.char_height))
        match = cv2.matchTemplate(self.digits_img, img, cv2.TM_CCOEFF)
        # print(self.digits_img.shape, img.shape, match.shape)
        _, _, _, (max_x, _) = cv2.minMaxLoc(match, None)
        # print(max_x, self.x_broders)
        # print_img(img)
        return self.x_to_digit(max_x + new_w // 2)

    def x_to_digit(self, x: int) -> int:
        for d, border in enumerate(self.x_broders):
            if x < border:
                return d
        raise ValueError(f"Invalid x coordinate: {x}")


def pairwise(coll: Iterable[T]) -> Iterable[Tuple[T, T]]:
    it = iter(coll)
    try:
        prev = next(it)
    except StopIteration:
        return
    while True:
        try:
            curr = next(it)
        except StopIteration:
            return
        yield prev, curr
        prev = curr


def print_img(img: np.ndarray):
    assert len(img.shape) == 2
    for line in img:
        print("".join(" " if v else "*" for v in line))


def get_contour_rects(img: np.ndarray) -> list[Rect]:
    conts, _  = cv2.findContours(img, cv2.RETR_EXTERNAL, cv2.CHAIN_APPROX_SIMPLE)
    return [Rect(*cv2.boundingRect(cont)) for cont in conts]


def get_number_areas(img: np.ndarray) -> Tuple[Rect, Rect]:
    s_img = cv2.cvtColor(img, cv2.COLOR_BGR2HSV)[:, :, 1]
    s_img = cv2.dilate(s_img, np.ones((10, 10)))
    s_img = cv2.erode(s_img, np.ones((8, 8)))
    _, s_img = cv2.threshold(s_img, 40, 255, cv2.THRESH_BINARY)

    rects = get_contour_rects(s_img)
    if len(rects) != 2:
        raise RuntimeError(f"Cannot determine number areas, got {len(rects)} contours")
    rects.sort(key=lambda r: r.w)
    row_numbers_rect, col_numbers_rect = rects
    return row_numbers_rect, col_numbers_rect


def extract_rect(img: np.ndarray, rect: Rect) -> np.ndarray:
    return img[rect.y:rect.y + rect.h, rect.x:rect.x + rect.w]


def cluster_1d(vec: list[int]) -> dict[int, int]:
    max_gap = max(x2 - x1 for x1, x2 in pairwise(vec))
    result = {vec[0]: 0}
    cluster_idx = 0
    prev = vec[0]
    for x in vec[1:]:
        if (x - prev) * 2 > max_gap:
            cluster_idx += 1
        result[x] = cluster_idx
        prev = x
    return result


def find_number_rects(img: np.ndarray, direction: Direction) -> list[list[[Rect]]]:
    def line_coord(r: Rect):
        return r.x if direction is Direction.cols else r.y

    def other_coord(r: Rect):
        return r.x if direction is Direction.rows else r.y

    _, img = cv2.threshold(img, 0xB0, 0xFF, cv2.THRESH_BINARY)
    conts, _  = cv2.findContours(img, cv2.RETR_EXTERNAL, cv2.CHAIN_APPROX_SIMPLE)
    rects: list[Rect] = []
    for cont in conts:
        rect = Rect(*cv2.boundingRect(cont))
        if cv2.contourArea(cont) and 0.9 < rect.w / rect.h < 1.1 and rect.w * rect.h / cv2.contourArea(cont) < 1.2:
            rects.append(rect)
        else:
            print("Skipped contour", cv2.contourArea(cont), rect.w / rect.h, rect.w * rect.h / (cv2.contourArea(cont) + 0.1))

    line_map = cluster_1d(sorted({line_coord(r) for r in rects}))

    rects.sort(key=lambda r: (line_map[line_coord(r)], other_coord(r)))

    result = [list(grp) for _, grp in groupby(rects, lambda r: line_map[line_coord(r)])]
    line_len = len(result[0])
    for idx, line in enumerate(result[1:], 1):
        if len(line) != line_len:
            raise RuntimeError(f"Line {idx} has length {len(line)}, not {line_len}")
    return result


def parse_cell(img: np.ndarray, ocr: DigitOCR, debug: bool) -> Optional[int]:
    rects = get_contour_rects(img)
    if not rects:
        return None
    rects.sort(key=lambda r: r.x)
    result = 0
    for rect in rects:
        digit_img = extract_rect(img, rect)
        result = result * 10 + ocr.recognize(digit_img)
    return result


def parse_numbers(img: np.ndarray, direction: Direction, ocr: DigitOCR) -> list[list[int]]:
    img = cv2.cvtColor(img,  cv2.COLOR_BGR2GRAY)
    rects = find_number_rects(img, direction)
    _, img = cv2.threshold(img, 127, 255, cv2.THRESH_BINARY_INV)
    result = []
    for l_idx, line in enumerate(rects):
        result_line = []
        for r_idx, rect in enumerate(line):
            value = parse_cell(extract_rect(img, rect), ocr, l_idx == 23 and r_idx == 5)
            if value is not None:
                result_line.append(value)
        result.append(result_line)
    return result


def read_image(fname: Optional[str]):
    ocr = DigitOCR(os.path.join(os.path.dirname(__file__), "digits.png"))
    pil_img: Image.Image
    if fname:
        pil_img = Image.open(fname)
    else:
        pil_img = ImageGrab.grabclipboard()
    img = cv2.cvtColor(np.array(pil_img), cv2.COLOR_RGB2BGR)
    row_numbers_rect, col_numbers_rect = get_number_areas(img)

    return {
        "row_hints": parse_numbers(extract_rect(img, row_numbers_rect), Direction.rows, ocr),
        "col_hints": parse_numbers(extract_rect(img, col_numbers_rect), Direction.cols, ocr),
    }
