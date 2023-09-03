import os
import random as rnd
from math import sin
from math import cos as cosine

glob = 5

def func(param1, paramnot2, param3=5):
    for j in range(2):
        print(j)
    for i in range(param3):
        print(j)
    
    k = os.listdir(".")
    print(sin(param1 + rnd.randint(1, 10)) + cosine(param3))
    
    while i < 5:
        print(i)
        i += 1
    
    k = 4
    if k == 0:
        j = 5
    elif k == 1:
        j = 4
    else:
        j = 2
    
    print(j)

def func2(p1, p2, *args, **kwargs):
    print("p1:", p1)
    print("p2:", p2)
    print(args)
    print(kwargs)

if __name__ == "__main__":
    func(param1=5, paramnot2=7)
    func2(1, 2, 3, 4, g="gg", h=600)
