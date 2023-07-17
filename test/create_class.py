class Rect(Shape):  

    STATIC_A = 5
    
    def __init__(self, a=STATIC_A, b=5):
        self.a=a
        self.b=b+1
    
    STATIC_B=6     
    ANOTHER_STATIC     =     5         
    
    def func2(self, a, b, c=2):  
        self.c = self.a * a + self.b * b + c
        print("Banana")

    MORE_STATIC="Static string"