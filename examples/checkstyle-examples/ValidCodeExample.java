package com.example;

import java.util.List;
import java.util.Map;
import java.util.HashMap;

public class ValidCodeExample {
    private static final int MAX_SIZE = 100;
    private List<String> items;
    
    public ValidCodeExample() {
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
                System.out.println("Invalid item: " + e.getMessage());
            } catch (Exception e) {
                System.out.println("Error: " + e.getMessage());
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
            default:
                System.out.println("Multiple");
                break;
        }
        
        int a = 1;
        int b = 2;
        
        if (item != null) {
            System.out.println(item);
        }
    }
    
    public boolean isValid(String value) {
        return value != null && !value.isEmpty();
    }
}

