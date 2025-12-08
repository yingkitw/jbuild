package com.example;

import java.util.List;
import java.util.ArrayList;
import java.util.Map;

public class ComplexExample {
    private static final int MAX_SIZE = 100;
    private List<String> items;
    
    public ComplexExample() {
        this.items = new ArrayList<>();
    }
    
    public void processItems(List<String> input) {
        if (input == null || input.isEmpty()) {
            return;
        }
        
        for (String item : input) {
            try {
                processItem(item);
            } catch (IllegalArgumentException e) {
                // Empty catch - should trigger violation
            } catch (Exception e) {
                System.out.println("Error: " + e.getMessage()); // Not empty
            }
        }
    }
    
    private void processItem(String item) {
        switch (item.length()) {
            case 0:
                System.out.println("Empty");
                break;
            case 1:
                System.out.println("Single");
                break;
            // Missing default - should trigger violation
        }
        
        int a = 1, b = 2; // Multiple declarations - should trigger violation
        
        if (item != null) {
            ; // Empty statement - should trigger violation
        }
    }
    
    public boolean isValid(String value) {
        if (value != null && !value.isEmpty()) {
            return true;
        } else {
            return false; // Can be simplified
        }
    }
}

