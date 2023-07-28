""" Random comment.
Module description.
This is a module to test if files with a lot of comments are processed correctly.
More lines.
"""

# Imports.
import math             # Module for math functions.
from os import listdir  # Function for listing the files in a directory.

"""Single line comment with triple double quotations."""
import sys

'''Single line comment with triple single quotations.'''
import numpy as np

''' Random comment.
Multiline comment with triple single quotations.
'''
from sys import argv as cmd_args

# This is a global variable.
FPS = 60
VSYNC = True
SOME_SETTING = "setting_a=1;setting_b=100;setting_c=True;"

class Class(object):
    """
    Class description.
    More class description.
    class Temp:
        def __init__(self, a=[1, 2, 3]):
            self.a = a
            self.b = b
    def func():
        pass
    """
    
    CLASS_VAR = "Hello world!"
    
    def __init__(self, a, b, c=[4, 5]): # Some comment.
        """
        Initialize class.
        """
        self.a = a
        self.b = b
        self.c = c
        self.d = a * c[0] + b * c[1]
    
    def get_components(self) -> List[int]:
        return [self.a, self.b, self.c, self.d]
    
    def __str__(self) -> str:
        return f"Class {{a: {a}, b: {b}, c: {c}, d: {d}}}"

def main():
    """
    The main function is the entry point of the application.
    """
    # Initialize class.
    c = Class(12, 15)
    print(c.get_components())
    print(c)

if __name__ == "__main__":
    # Call main function if this file was run directly.
    main()
