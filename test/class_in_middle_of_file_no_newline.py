import math
import random as rnd
from os import listdir

SETTING = "Banana"
class Mango(Fruit):
    
    CLASSVAR = "MangoFruit"
    
    def __init__(self, size):
        super().__init__("Mango")
        self.size = size
    
    def get_size(self):
        return self.size
    
    def print_size(self):
        print(f"Fruit size is: {self.size}")
def main(fruit_size):
    fruit = Mango(fruit_size)
