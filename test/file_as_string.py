import math, random as rnd
from os import listdir
from fruits import apple as a, banana as b, mango as m

FPS = 60        # Frames per second
VSYNC = True    # Vertical sync

class Rect:
    
    def __init__(self, a):
        self.a = a

def function(p1, p2='5'):
    print(p1, p2)

if __name__ == "__main__":
    function(Rect(a))