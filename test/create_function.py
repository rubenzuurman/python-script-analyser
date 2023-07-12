def func_name(param1, param2, param3=5, *args, **kwargs):
    Appel
    for i in range(100):
        print(i + 5 * 10)
        if i % 5 == 0:
            print(f"{i} is divisible by 5")
        else:
            print("no")
            if i % 7 == 0:
                print(f"{i} is divisible by 7")
    
    Banaan