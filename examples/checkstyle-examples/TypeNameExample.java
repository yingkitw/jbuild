package com.example;

public class invalidClassName { // Invalid - should start with uppercase - should trigger violation
}

class anotherInvalidName { // Invalid - should start with uppercase
}

public class ValidClassName { // Valid - starts with uppercase
}

interface InvalidInterfaceName { // Invalid - should start with uppercase
}

interface ValidInterfaceName { // Valid - starts with uppercase
}

enum invalidEnumName { // Invalid - should start with uppercase
    VALUE1, VALUE2
}

enum ValidEnumName { // Valid - starts with uppercase
    VALUE1, VALUE2
}

