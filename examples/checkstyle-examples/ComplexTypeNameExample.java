package com.example;

public class ComplexTypeNameExample {
    // Valid class names
    public class ValidClass {}
    public class AnotherValidClass {}
    
    // Invalid class names
    class invalidClass {} // Should trigger violation
    public class Invalid_Class {} // Should trigger violation if format doesn't allow underscore
    public class invalidClass2 {} // Should trigger violation
    
    // Interfaces
    public interface ValidInterface {}
    interface invalidInterface {} // Should trigger violation
    
    // Enums
    public enum ValidEnum { VALUE1, VALUE2 }
    enum invalidEnum { VALUE1, VALUE2 } // Should trigger violation
    
    // Inner classes
    public static class ValidInnerClass {}
    static class invalidInnerClass {} // Should trigger violation
}

