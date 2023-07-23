


class Banana(Fruit, Yellow, object):
    
    CLASS_VAR_1 = "500 is not equal to 100"
    
    def __init__(self, size):
        super().__init__()
        self.sub_func_ran = False
        
        def sub_func(a, b):
            self.sub_func_ran = True
            return a * b + 5
        
        self.size = size
    
    SETTING = True
    
    class SubClass(Building):
        
        def __init__(self, height) -> Self:
            super().__init__()
            
            self.height = height
        
        def get_height(self) -> int:
            return self.height

