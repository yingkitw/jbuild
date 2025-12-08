package com.example;

public class MissingSwitchDefaultExample {
    public void method1(int value) {
        switch (value) {
            case 1:
                System.out.println("One");
                break;
            case 2:
                System.out.println("Two");
                break;
            // Missing default case - should trigger violation
        }
    }

    public void method2(String str) {
        switch (str) {
            case "A":
                return;
            case "B":
                return;
            // Missing default case - should trigger violation
        }
    }

    public void method3(int value) {
        switch (value) {
            case 1:
                break;
            case 2:
                break;
            default:
                System.out.println("Other"); // Has default - should not trigger violation
                break;
        }
    }
}

