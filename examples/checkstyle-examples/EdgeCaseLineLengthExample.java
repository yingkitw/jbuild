package com.example;

public class EdgeCaseLineLengthExample {
    // Line with exactly 80 characters should be fine
    // This line is 81 characters long and should trigger a violation
    public void method1() {
        String s = "This is a very long string literal that makes the line exceed 80 characters";
    }

    public void method2() {
        // Line with tabs:		This should be expanded and checked
        int x = 5;
    }

    // Package and import statements should be ignored by default
    // But this comment line is too long and should trigger a violation if not ignored
}

