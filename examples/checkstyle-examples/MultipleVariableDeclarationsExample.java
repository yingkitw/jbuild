package com.example;

public class MultipleVariableDeclarationsExample {
    public void method1() {
        int a = 1, b = 2, c = 3; // Multiple declarations - should trigger violation
    }

    public void method2() {
        String x = "hello", y = "world"; // Multiple declarations - should trigger violation
    }

    public void method3() {
        int a = 1; // Single declaration - should not trigger violation
        int b = 2;
    }

    public void method4() {
        for (int i = 0, j = 0; i < 10; i++) { // Multiple in for init - should trigger violation
            System.out.println(i);
        }
    }
}

