// examples/stderr_patterns.rs
// Test stderr usage patterns as they appear in BookDB codebase
// Run with: cargo run --example stderr_patterns

use stderr::Stderr;

fn main() {
    println!("🧪 Testing BookDB Stderr Usage Patterns");
    println!("=======================================\n");
    
    // Check stderr version compatibility
    check_stderr_version();
    
    test_bookdb_logging_patterns();
    test_bookdb_trace_patterns();
    test_bookdb_format_patterns();
    test_bookdb_debug_patterns();
    test_bookdb_interactive_patterns();
    test_bookdb_context_patterns();
    test_bookdb_error_scenarios();
    test_realistic_scenarios();
    
    println!("\n🎉 Stderr pattern testing complete!");
}

fn check_stderr_version() {
    println!("🔍 Checking Stderr Version Compatibility");
    
    // Get stderr version at runtime
    let stderr_version = stderr::VERSION;
    println!("📦 Stderr version: {}", stderr_version);
    
    // Check if we're using a compatible version (0.8.x)
    if stderr_version.starts_with("0.8.") {
        println!("✅ Compatible stderr version (0.8.x series)");
    } else {
        println!("⚠️  WARNING: This test was designed for stderr 0.8.x series");
        println!("   Current version: {}", stderr_version);
        println!("   Some tests may fail with different API versions");
    }
    
    // Test basic logger creation to verify it works
    let mut logger = stderr::Stderr::new();
    logger.info("Version check: stderr logger created successfully");
    
    println!("✅ Stderr version check complete\n");
}

fn test_bookdb_logging_patterns() {
    println!("📝 Testing Basic Logging Patterns");
    
    let mut logger = Stderr::new();
    
    // Test basic logging (used throughout BookDB)
    logger.info("BookDB starting up");
    logger.warn("Configuration file not found, using defaults");
    logger.error("Database connection failed");
    logger.okay("Context chain validated successfully");
    
    println!("✅ Basic logging patterns work\n");
}

fn test_bookdb_trace_patterns() {
    println!("🔍 Testing Trace Patterns");
    
    let mut logger = Stderr::new();
    
    // Test trace_fn pattern (used in V3 validator, database operations)
    logger.trace_fn("v3_validator", "starting context validation");
    logger.trace_fn("database", "connecting to SQLite");
    logger.trace_fn("context_manager", "loading cursor state");
    
    // Test regular trace
    logger.trace("Regular trace message");
    
    println!("✅ Trace patterns work\n");
}

fn test_bookdb_format_patterns() {
    println!("📝 Testing Format Patterns");
    
    let mut logger = Stderr::new();
    
    // These are the patterns BookDB actually uses that need format strings
    let variable_count = 42;
    let context = "@webapp.config.var.secrets";
    let operation = "set_variable";
    
    // Pattern 1: Simple format with single value (very common in BookDB)
    let msg1 = format!("Processing {} variables", variable_count);
    logger.info(&msg1);
    
    // Pattern 2: Context chain logging (used throughout BookDB)
    let msg2 = format!("Resolved context: {}", context);
    logger.info(&msg2);
    
    // Pattern 3: Database operation logging (common in database layer)
    let msg3 = format!("database: {} completed for context: {}", operation, context);
    logger.trace(&msg3);
    
    // Pattern 4: trace_fn with formatted message (used in V3 validator)
    let input_chain = "@work@proj.workspace.var.keystore";
    let msg4 = format!("validating context chain: '{}'", input_chain);
    logger.trace_fn("v3_validator", &msg4);
    
    // Pattern 5: Multi-value formatting (used in import/export)
    let file_path = "config.env";
    let key_count = 15;
    let msg5 = format!("Imported {} variables from {}", key_count, file_path);
    logger.okay(&msg5);
    
    println!("✅ Format patterns work with manual format!()\n");
}

fn test_bookdb_debug_patterns() {
    println!("🐛 Testing Debug Patterns");
    
    let mut logger = Stderr::new();
    
    // Test debug value logging (used for structs and complex data)
    #[derive(Debug)]
    #[allow(dead_code)]  // Fix the dead code warning
    struct ContextChain {
        base: String,
        project: String,
        workspace: String,
        anchor: String,
    }
    
    let context = ContextChain {
        base: "work".to_string(),
        project: "webapp".to_string(),
        workspace: "config".to_string(),
        anchor: "var".to_string(),
    };
    
    // Test debug logging
    logger.info_debug(&context);
    logger.trace_debug(&context);
    
    // Test inspect interface
    logger.inspect().info(&context);
    
    println!("✅ Debug patterns work\n");
}

fn test_bookdb_interactive_patterns() {
    println!("💬 Testing Interactive Patterns");
    
    let mut logger = Stderr::new();
    
    // BookDB uses banners for section headers
    if let Ok(_) = logger.banner("Context Chain Validation", '=') {
        println!("✅ Banner works");
    } else {
        println!("❌ Banner failed");
    }
    
    // BookDB uses boxes for important messages
    if let Ok(_) = logger.box_light("Critical: Database schema migration required") {
        println!("✅ Box light works");
    } else {
        println!("❌ Box light failed");
    }
    
    // Note: Skip confirm() in examples as it waits for input
    // logger.confirm("Continue with migration?")?;
    
    println!("✅ Interactive patterns work (confirm skipped)\n");
}

fn test_bookdb_context_patterns() {
    println!("🎯 Testing Context Patterns");
    
    let mut logger = Stderr::new();
    
    // BookDB sets context for operations
    logger.set_context("@webapp.VAR.config");
    logger.info("Working in webapp config context");
    
    // Same context shouldn't show banner again
    logger.set_context("@webapp.VAR.config");
    logger.info("Still in same context");
    
    // Different context should show new banner
    logger.set_context("@api.VAR.secrets");
    logger.warn("Switched to API secrets context");
    
    // Clear context
    logger.clear_context();
    logger.info("Back to neutral context");
    
    println!("✅ Context patterns work\n");
}

fn test_bookdb_error_scenarios() {
    println!("⚠️ Testing Error Scenarios");
    
    let mut logger = Stderr::new();
    
    // Test error reporting patterns used in BookDB
    let error_msg = "Failed to parse context chain";
    let invalid_input = "invalid@chain@format";
    let detailed_error = format!("{}: '{}'", error_msg, invalid_input);
    logger.error(&detailed_error);
    
    // Test validation error patterns
    let field_name = "project_name";
    let invalid_value = "invalid-chars!";
    let validation_error = format!("Invalid {}: '{}'", field_name, invalid_value);
    logger.warn(&validation_error);
    
    // Test success confirmation patterns
    let operation = "migration";
    let record_count = 156;
    let success_msg = format!("{} completed successfully: {} records processed", operation, record_count);
    logger.okay(&success_msg);
    
    println!("✅ Error scenario patterns work\n");
}

fn test_realistic_scenarios() {
    println!("📋 Testing Realistic BookDB Scenarios");
    
    test_context_validation_scenario();
    test_database_operation_scenario();
    test_import_operation_scenario();
}

fn test_context_validation_scenario() {
    println!("  🔍 Context Validation Scenario");
    
    let mut logger = Stderr::new();
    
    // Simulate V3 context validation workflow
    if let Ok(_) = logger.banner("Context Chain Validation", '-') {
        let input = "@work@webapp.config.var.secrets";
        logger.trace_fn("v3_validator", &format!("validating input: '{}'", input));
        
        logger.trace_fn("v3_validator", "parsing prefix mode");
        logger.trace_fn("v3_validator", "validating base component");
        logger.trace_fn("v3_validator", "parsing main components");
        
        logger.okay("Context chain validation successful");
    }
    
    println!("  ✅ Realistic validation scenario works");
}

fn test_database_operation_scenario() {
    println!("  🗄️ Database Operation Scenario");
    
    let mut logger = Stderr::new();
    
    logger.set_context("@webapp.VAR.config");
    
    let key = "DATABASE_URL";
    let value = "sqlite:///app.db";
    
    logger.trace_fn("database", &format!("setting variable '{}' in context", key));
    logger.trace_fn("database", "ensuring project namespace exists");
    logger.trace_fn("database", "ensuring keystore namespace exists");
    logger.trace_fn("database", &format!("inserting/updating variable: {} = {}", key, value));
    
    logger.okay(&format!("Variable '{}' set successfully", key));
    
    println!("  ✅ Realistic database scenario works");
}

fn test_import_operation_scenario() {
    println!("  📦 Import Operation Scenario");
    
    let mut logger = Stderr::new();
    
    let file_path = "config.env";
    let variable_count = 23;
    
    if let Ok(_) = logger.banner("Import Operation", '=') {
        logger.info(&format!("Starting import from: {}", file_path));
        
        logger.trace_fn("import", "parsing file format");
        logger.trace_fn("import", &format!("found {} variables to import", variable_count));
        logger.trace_fn("import", "validating variable names");
        logger.trace_fn("import", "starting database transaction");
        
        for i in (1..=variable_count).step_by(10) {
            logger.trace(&format!("imported {} variables", i));
        }
        
        logger.okay(&format!("Import completed: {} variables imported from {}", variable_count, file_path));
    }
    
    println!("  ✅ Realistic import scenario works\n");
}
