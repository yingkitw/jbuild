package com.example.api;

import com.example.core.StringUtils;

/**
 * Greeter API interface.
 */
public interface Greeter {
    
    /**
     * Generate a greeting message.
     */
    String greet(String name);

    /**
     * Default implementation that capitalizes the name.
     */
    default String greetCapitalized(String name) {
        return greet(StringUtils.capitalize(name));
    }
}
