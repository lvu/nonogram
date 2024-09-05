# Nonogram OCR and solver

My hobby project for solving [nonograms](https://en.wikipedia.org/wiki/Nonogram).
Doesn't solve _all_ possible nonograms, but isn't limited to line-by-line solving either.

There is also a sub-project that recognizes nonograms from the [Katana](https://nonograms-katana.com/) website.

## Installation

    python3 -m venv venv
    ./venv/bin/pip install -r requirements.txt
    . ./venv/bin/activate

## Usage
To solve a nonogram, feed a json describing it to the solver:
    
    cat nonogram.json | python main.py solve

To recognize a nonogram, first take a screenshot from the Katana website and crop it just outsude the black border.
Then run 

    python main.py nonogram.png > nonogram.json
    
You can also recognize from clipboard on Mac (and probably on Windows); in this case, don't specify the input file.
