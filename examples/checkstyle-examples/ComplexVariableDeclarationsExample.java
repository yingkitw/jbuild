package com.example;

public class ComplexVariableDeclarationsExample {
    // Field declarations - should not trigger (check is for local variables)
    private int a = 1, b = 2;
    
    public void method1() {
        // Multiple declarations - should trigger violation
        int x = 1, y = 2, z = 3;
    }

    public void method2() {
        // Single declarations - should not trigger violation
        int x = 1;
        int y = 2;
        int z = 3;
    }

    public void method3() {
        // Multiple declarations with initialization
        String s1 = "hello", s2 = "world";
    }

    public void method4() {
        // For loop with multiple declarations in init - should trigger violation
        for (int i = 0, j = 0; i < 10; i++) {
            System.out.println(i);
        }
    }

    public void method5() {
        // For loop with single declaration - should not trigger violation
        for (int i = 0; i < 10; i++) {
            System.out.println(i);
        }
    }

    public void method6() {
        // Multiple declarations on same line but different statements
        int a = 1;
        int b = 2; // Should not trigger violation
    }
}

