// using_stderr_current.rs - Quick test of current stderr capabilities
use stderr::Stderr;


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


fn main() {
    let mut logger = Stderr::new();


    check_stderr_version();
    
    // Basic logging (should work)
    logger.info("Basic info message");
    logger.warn("Basic warning");
    logger.error("Basic error");
    logger.okay("Success message");
    
    // Feature-gated methods (should work with default features)
    logger.trace("Trace message");
    logger.debug("Debug message");
    
    // Trace function (should work with trace feature)
    logger.trace_fn("test_function", "testing trace_fn method");
    
    // Formatting (should work with formatting feature)
    if let Ok(_) = logger.banner("Test Banner", '=') {
        println!("Banner method works!");
    }
    
    // Interactive (should work with interactive feature)  
    let _ = logger.confirm("Test confirmation?"); // Skip in automated test
    
    // Debug values
    let test_data = vec![1, 2, 3];
    logger.info_debug(&test_data);

}
