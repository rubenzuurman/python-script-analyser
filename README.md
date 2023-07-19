# Readme

### Goal
1. Goal of the project
    - Analyse a python script.
    - Notify the user when an import is not used.
    - Notify the user when a function is never called.
    - Notify the user when a refenced variable is either not defined or 'out of scope'. This doesn't give an error but sometimes leads to undesired behaviour. This is the primary function of the program.
    - Notify the user of inconsistent indentation.
    - Notify the user of usage of global variables without using 'global <variable\>' in the function.
    - Classes inside of classes.
    - Classes inside of functions.
2. Things that are not going to be supported
    - Checking for invalid variable names/function names/class names (invalid names will be skipped).
    - Multiline assignments.

### General notes

12-07-2023<br />
Maybe pretend the entire file (except the root) is a function, and handle the file as a function (aka get list of functions, imports, classes, variables), but handle it as a root case (the global keyword can be used in this function). Then analyse any subfunction the same way (but without root), and any class can be handled with its own handling function (which should only be slightly different). This way you can create a nested analysis, in which each object deeper has its own scope.
Also create warning codes so the user can suppress warnings that they don't want to see. (Be careful not to create a linter, those already exist.)<br />

15.56<br />
Pushed local git repository to remote github repository.

22.24<br />
Properties of each type of python object (file is entire file, function is function, class is class):

1. File
    - name
    - parameters (commandline)
    - imports
    - global variables
    - functions
    - classes
    - roots
2. Function
    - name
    - parameters
    - imports
    - variables defined in scope
    - subfunctions
    - subclasses
3. Class
    - name
    - parent
    - static class variables
    - class instance variables
    - functions
    - classes

13-07-2023<br />
14.34<br />
I don't need the roots field in the File struct which could be used to track the callstack, because any callstack issues are handled by the python interpreter.

15-07-2023<br />
00.18<br />
The current problem is that the regex pattern for class variables in class returns false positives on variables defined in class methods. I'll be fixing this tomorrow.

17-07-2023<br />
15.40<br />
The file processing functionality is now entirely contained in the lib.rs file, main.rs looks a lot cleaner now.

19-07-2023<br />
14.49<br />
I don't have a clear scope for the project right now. At the moment I have implemented reading in a file, extracting functions and classes, and extracting classes inside of classes. I am working on a struct called 'Assignment' in an effort to detect when variables are defined and what they are called. At the end of the day the goal of this project was to detect when a variable is used out of scope. For example when there is a for loop looping over list indices with the looping variable called index, and index is referenced after the for loop, python does not throw an error, but this may be undesirable.

15.04<br />
Variable assignments are supported in this exact way:<br />
&emsp;```^[\t ]*(?P<var>\w+)[\t ]*(:.*)?=[\t ]*(?P<val>.*)$```<br />
This allows for type hinting when assigning a variable. This is only recognized as an assignment if any equal signs in the part after the first equal sign are enclosed in either of the following:<br />
&emsp;'', "", (), [], {}

15.22<br />
I am currently deciding that dictionary key validity on assignment will not be checked.

21.47<br />
I have implemented the Assignment struct today and changed the type of class variables from String to Assignment.

22.53<br />
I have reimplemented the Class::get_source() method, it now reconstructs the source from the functions, classes, and variables inside the class.
