package com.example;

/**
 * Simple Hello World application
 */
public class App {
    private String message;
    
    public App() {
        this.message = "Hello, World!";
    }
    
    public App(String message) {
        this.message = message;
    }
    
    public String getMessage() {
        return message;
    }
    
    public void setMessage(String message) {
        this.message = message;
    }
    
    public static void main(String[] args) {
        App app = new App();
        System.out.println(app.getMessage());
    }
    
    public String greet(String name) {
        return "Hello, " + name + "!";
    }
}

