import math

def sqrt_bulk(numbers):
    def sqrt(x):
        return math.sqrt(x)
    
    for n in numbers:
        yield sqrt(n)

def cube_bulk(numbers):
    def cube(x):
        def square(x):
            return x * x
        return square(x) * x
    
    for n in numbers:
        yield cube(n)

def main():
    print("Input 'q' to start calculating.")
    numbers = []
    while True:
        inp = input()
        if inp == "q":
            break
        try:
            n = int(inp)
            numbers.append(n)
        except ValueError as e:
            print(f"Cannot cast '{inp}' to int.")
    
    for n, result in zip(numbers, cube_bulk(numbers)):
        print(f"{n}**3 = {result}")

if __name__ == "__main__":
    main()