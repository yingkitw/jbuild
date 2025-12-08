package com.example;

public class NestedEmptyCatchExample {
    public void method1() {
        try {
            try {
                doSomething();
            } catch (Exception e) {
                // Nested empty catch - should trigger violation
            }
        } catch (Exception e) {
            System.out.println("Outer catch");
        }
    }

    public void method2() {
        try {
            doSomething();
        } catch (IllegalArgumentException e) {
            // Empty catch
        } catch (NullPointerException e) {
            // Another empty catch
        } catch (Exception e) {
            System.out.println("Last catch");
        }
    }
}

