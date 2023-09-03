def func_if_elif_else():
    for a in range(5):
        print(a)
    if a > 4:
        print(a)
    elif b == 4:
        print(b)
    else:
        print(c)

def func_with():
    with open("test/file.txt") as file:
        print(file)
    if file.is_open:
        print("Open")
    
    with socket.accept("127.0.0.1", 12234) as conn:
        conn.connect("abc")