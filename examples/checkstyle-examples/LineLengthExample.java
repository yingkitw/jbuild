package com.example;

public class LineLengthExample {
    // This line is exactly 80 characters long and should be fine
    // This line is way too long and exceeds the maximum line length limit of 80 characters by a significant margin
    public void method1() {
        String veryLongVariableName = "This is a very long string that makes the line exceed the maximum length";
    }

    public void method2() {
        // This is a normal line that should be fine
        int x = 5;
    }
}

