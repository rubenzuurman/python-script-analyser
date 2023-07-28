class Rect(object): # This class is for checking if these type of comments get ignored.
    
    """
    This is a class description, it should be ignored.
    def __init__(self, a=5):
        print("This is some function definition in a class description.")
    class Sub:
        def __init__(self, b=5):
            print("Class definition in class description")
    GLOBAL_VAR = 5
    """
    """This is a single line multiline comment."""
    
    # Some comment.
    GLOBAL_VARIABLE = 6
    # Commented_variable = 5
    SOME_VAR = "Banaan" # Comment a=5.
    
    def __init__(self, a=5, b=GLOBAL_VARIABLE, c=8): # Banaan.
        """
        This is a function description.
        """
        self.a = a # Foo
        self.b = b # Bar
        print(f"a * b: {a * b}") # Baz
