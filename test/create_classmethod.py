    def class_method(self, a, b=10, c=5, d = 15, *args, **kwargs):  
        self.a = a
        self.b = b + 5
        self.c = c + 10
        if "c_extra" in kwargs:
            self.c += kwargs["c_extra"]
        else:
            self.c += 7