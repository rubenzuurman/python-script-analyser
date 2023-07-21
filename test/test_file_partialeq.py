import math, random as rnd

from os import listdir

PRINT_OR_NO = True

class PrintClass(object):
    
    CLASS_ID = 0
    
    def __init__(self, var1=500):
        global PRINT_OR_NO
        
        self.var1 = var1
        self.var2 = math.sqrt(var1)
        
        if PRINT_OR_NO:
            print(f"[ID{PrintClass.CLASS_ID}] var1: {var1}, var2: {var2}")
        
        PrintClass.CLASS_ID += 1
        print("End of init")
    
    def somefunc(self, var3):
        self.var3 = var3
        self.var4 = self.var1 * self.var2 + self.var3 * math.pow(self.var1, 5)

def main():
    printclass = PrintClass(var1=150)
    
    print("Banana")

if __name__ == "__main__":
    main()