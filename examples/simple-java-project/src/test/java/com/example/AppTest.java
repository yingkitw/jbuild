package com.example;

import org.junit.Test;
import static org.junit.Assert.*;

/**
 * Unit tests for App class
 */
public class AppTest {
    
    @Test
    public void testDefaultConstructor() {
        App app = new App();
        assertEquals("Hello, World!", app.getMessage());
    }
    
    @Test
    public void testParameterizedConstructor() {
        App app = new App("Test Message");
        assertEquals("Test Message", app.getMessage());
    }
    
    @Test
    public void testGetSetMessage() {
        App app = new App();
        app.setMessage("New Message");
        assertEquals("New Message", app.getMessage());
    }
    
    @Test
    public void testGreet() {
        App app = new App();
        assertEquals("Hello, Alice!", app.greet("Alice"));
        assertEquals("Hello, Bob!", app.greet("Bob"));
    }
}

