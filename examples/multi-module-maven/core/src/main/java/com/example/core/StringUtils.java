package com.example.core;

/**
 * String utility functions.
 */
public class StringUtils {

    /**
     * Reverse a string.
     */
    public static String reverse(String input) {
        if (input == null) {
            return null;
        }
        return new StringBuilder(input).reverse().toString();
    }

    /**
     * Check if a string is empty or null.
     */
    public static boolean isEmpty(String input) {
        return input == null || input.isEmpty();
    }

    /**
     * Capitalize the first letter of a string.
     */
    public static String capitalize(String input) {
        if (isEmpty(input)) {
            return input;
        }
        return Character.toUpperCase(input.charAt(0)) + input.substring(1);
    }
}
