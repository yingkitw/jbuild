package com.example;

public class ComplexSwitchExample {
    public void method1(int value) {
        switch (value) {
            case 1:
                System.out.println("One");
                break;
            case 2:
                System.out.println("Two");
                break;
            // Missing default - should trigger violation
        }
    }

    public void method2(String str) {
        switch (str) {
            case "A":
                return;
            case "B":
                return;
            // Missing default - should trigger violation
        }
    }

    public void method3(int value) {
        switch (value) {
            case 1:
                break;
            default:
                System.out.println("Other"); // Has default - should not trigger violation
                break;
        }
    }

    public void method4(int value) {
        switch (value) {
            default:
                System.out.println("Default first"); // Has default - should not trigger violation
                break;
            case 1:
                break;
        }
    }

    public void method5() {
        switch (getValue()) {
            case 1:
            case 2:
            case 3:
                System.out.println("Multiple cases");
                break;
            // Missing default - should trigger violation
        }
    }

    private int getValue() {
        return 1;
    }
}

