def function(a=5, b=6, c=[5, 6, 7, 8.5], d={"a": 5, "b": 7}): # Some comment.
    """Single line multiline comment."""
    """
    Multiline multiline comment.
    More text.
    """
    
    # Single line comment.
    print(a, b, c, d)
    
    # Return something if 5 and 6.
    if a == 5 and b == 6:
        """Some more comments."""
        return True
    # Return something else if not 5 and 6.
    else:
        return False or c[3] == 8.5
    """
    A = 5
    """
    
    a = """
        This is a multiline string literal.
        Another line.
    """
