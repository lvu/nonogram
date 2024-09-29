# Nonogram solver and OCR

My hobby project for solving [nonograms](https://en.wikipedia.org/wiki/Nonogram).

There is also a sub-project that recognizes nonograms.

## File format

The solver uses and the OCR produces nonogram descriptions in a simple JSON format, which liiks like this:

    {
      "row_hints": [
        [1],
        [1]
      ],
      "col_hints": [
        [1],
        [1]
      ]
    }

All the hints are arranged the way they are in the usual nonogram: left-to-right and top-to-bottom.

## Solver

The solver is implemented in Rust. It uses branch and bound approach, together with an efficient line-by-line solving.

### Installation

Ensure you have Rust and Cargo installed, and execute

    cargo install --path .

You now can run it as `nono-solver`.

### Usage

    nono-solver [OPTIONS] [FNAME]

If FNAME is no specified, the input JSON will be read from _stdin_.

#### Options
* `-m, --max-depth <MAX_DEPTH>`: maximum depth of branching. 0 means no branching, solving solely line-by-line; it should be enough for "proper" nonograms.
Specifying a number larger then necessary doesn't hurt, as the solver tries lesser depths first.
* `-f, --find-all`: if this flag isn't specified, the solver terminates when it finds the first correct solution. Specify the flag if you want to find all the
solutions, or to check if the nonogram has only one solution.

## OCR

The OCR is written in Python and uses OpenCV library. Use it to avoid manually typing the nonogram descriptions,
if you want to just solve the puzzle you've found on a website.

Currently, ony the [Katana](https://nonograms-katana.com/) website is supported.

### Installation

Create and activate a Python virtualenv, if you want to. Then execute

    pip install -e .

You can now run it as `nono-ocr`.

### Usage

The solver takes an image from the first command-line argument, or, if unspecified, tries to read it from the clipboard.
Take a screenshot from the Katana website and crop it just outsude the black border. Then feed to to the OCR. The nonogram description
is produced to stdout; you can either save it so a file for later use, or feed it to the solver directly:

    nono-ocr sample.png | nono-solver