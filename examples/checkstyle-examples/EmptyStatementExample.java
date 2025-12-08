package com.example;

public class EmptyStatementExample {
    public void method1() {
        int x = 5;
        ; // Empty statement - should trigger violation
        System.out.println(x);
    }

    public void method2() {
        if (true) {
            ; // Empty statement in if block
        }
    }

    public void method3() {
        for (int i = 0; i < 10; i++) {
            ; // Empty statement in loop
        }
    }
}

