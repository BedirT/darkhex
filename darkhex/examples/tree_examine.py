"""
Sample file for using tree_run.py
"""
from darkhex.algorithms.tree_run import TreeRun


def main():
    file_name = "simplified_4x3_mccfr_p0"
    tree_run = TreeRun(file_name)
    tree_run.tree_run()


if __name__ == "__main__":
    main()
