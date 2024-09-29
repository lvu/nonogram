import sys

from .katana_source import read_image


def hints_repr(hints: list[list[int]]) -> str:
    return ",\n".join(
        f"    [{", ".join(str(hint) for hint in line_hints)}]"
        for line_hints in hints
    )


def main():
    data = read_image(sys.argv[1] if len(sys.argv) > 1 else None)
    row_hint_strs = ""
    print("{")
    print("  \"row_hints\": [")
    print(hints_repr(data["row_hints"]))
    print("  ],")
    print("  \"col_hints\": [")
    print(hints_repr(data["col_hints"]))
    print("  ]")
    print("}")