import math

class Rect:
    
    ID = 0
    TEMP_ID = ID
    
    def __init__(self, width, height):
        self.width = width
        self.height = height
        self.id = Rect.ID
        Rect.ID += 1
    
    def get_diagonal(self) -> float:
        return math.sqrt(self.width ** 2 + self.height ** 2)
    
    def print(self):
        print(f"Rect[width: {self.width}, height: {self.height}]")
    
    def misc(self):
        for i in range(5):
            print(i)
            for j in range(5):
                print(j)
            print(j)
            k = 5
            while k > 0:
                k -= 1
            print(k)
        print(i, j, k)

def main():
    rect = Rect(100, 200)
    print(rect.get_diagonal())
    rect.print()

if __name__ == "__main__":
    main()
