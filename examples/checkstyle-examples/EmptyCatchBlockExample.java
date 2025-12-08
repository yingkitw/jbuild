package com.example;

public class EmptyCatchBlockExample {
    public void method1() {
        try {
            doSomething();
        } catch (Exception e) {
            // Empty catch block - should trigger violation
        }
    }

    public void method2() {
        try {
            doSomething();
        } catch (IllegalArgumentException e) {
            // Also empty - should trigger violation
        }
    }

    public void method3() {
        try {
            doSomething();
        } catch (Exception e) {
            System.out.println("Handled"); // Not empty - should not trigger violation
        }
    }
}

