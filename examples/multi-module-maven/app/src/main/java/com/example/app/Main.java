package com.example.app;

import com.example.api.Greeter;

/**
 * Main application entry point.
 */
public class Main implements Greeter {

    @Override
    public String greet(String name) {
        return "Hello, " + name + "!";
    }

    public static void main(String[] args) {
        Main app = new Main();
        String name = args.length > 0 ? args[0] : "World";
        System.out.println(app.greetCapitalized(name));
    }
}
