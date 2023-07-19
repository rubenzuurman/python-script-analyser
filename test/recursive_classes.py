import math

SETTING = math.pow(math.sqrt(2), math.e * math.pi)

class UpperClass(object):
    BANANA = "Banana"
    class MiddleClass(Rect):
        
        def __init__(self):
            self.width = 5
            self.height = 10
        
        class LowerClass(Shape, Banana):
            
            LOWER_GLOB = "LowerClass class variable"
            SOME_OTHER_THING = "Apple"
            
            def __init__(self, banana, apple):
                self.banana = banana
                self.apple = apple
                self.mango = (banana * apple) / math.sqrt(2)
            
            def pear(self, orange):
                return self.apple * self.banana * orange
        
        def get_width(self, pineapple=25):
            return self.width
    
    def __init__(self, a, b):
        def define_c():
            self.c = 5
        
        define_c()
        self.a = [a, b, self.c + 1]
        self.b = 56
    
    def print(self):
        print(self.a, self.b, self.c)

def main():
    upper = UpperClass(5, 6)
