package Com.Example; // Invalid package name

import java.lang.String; // Redundant import
import java.lang.Integer; // Redundant import
import com.example.SomeClass; // Same package import
import java.util.List;
import java.util.List; // Duplicate import

public class invalidClassName { // Invalid class name
    public void method1() {
        int a = 1, b = 2, c = 3; // Multiple declarations
        ; // Empty statement
    }

    public void method2() {
        switch (value) {
            case 1:
                break;
            // Missing default
        }
    }

    public void method3() {
        try {
            doSomething();
        } catch (Exception e) {
            // Empty catch
        }
    }

    // This is a very long line that exceeds the maximum line length limit of 80 characters by a significant margin and should trigger a violation
}

