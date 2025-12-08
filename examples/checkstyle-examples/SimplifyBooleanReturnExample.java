package com.example;

public class SimplifyBooleanReturnExample {
    public boolean method1(boolean value) {
        if (value) {
            return true;
        } else {
            return false; // Can be simplified to: return value;
        }
    }

    public boolean method2(boolean value) {
        if (value) {
            return false;
        } else {
            return true; // Can be simplified to: return !value;
        }
    }

    public boolean method3(boolean value) {
        return value; // Already simplified - should not trigger violation
    }
}

