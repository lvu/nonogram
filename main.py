import sys

from nonogram.katana_source import read_image


if __name__ == '__main__':
    if len(sys.argv) > 1:
        read_image(sys.argv[1])
    else:
        read_image(None)
