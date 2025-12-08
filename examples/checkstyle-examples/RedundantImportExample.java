package com.example;

import java.lang.String; // Redundant - java.lang is automatically imported
import java.lang.Integer; // Redundant - java.lang is automatically imported
import com.example.PackageNameExample; // Redundant - same package
import java.util.List;
import java.util.List; // Duplicate import - should trigger violation

public class RedundantImportExample {
    public void method() {
        List<String> list = new ArrayList<>();
    }
}

