// tests/integration/phase1_tests.rs - Comprehensive tests for Phase 1 implementation
//
// TESTS:
// 1. CONCEPTS.md-compliant context parsing
// 2. Installation guard functionality
// 3. Context atomicity enforcement
// 4. CDCC and FQCC resolution
// 5. Stderr integration and context banners

#[cfg(test)]
mod phase1_integration_tests {
    use bookdb::{
        context::{parse_context_chain, DefaultResolver, ContextChain, Anchor, ChainMode, CursorState},
        installation::{InstallationGuard, InstallationManager},
        context_manager::ContextManager,
        config::Config,
        error::{Result, BookdbError},
    };
    use tempfile::TempDir;
    use std::path::PathBuf;
    
    /// Helper to create isolated test environment
    fn create_test_environment() -> (Config, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let mut config = Config::default();
        config.xdg = bookdb::config::XdgDirs::new(temp_dir.path()).unwrap();
        (config, temp_dir)
    }
    
    #[test]
    fn test_concepts_md_compliant_parsing() -> Result<()> {
        // Test FQCC: work@website.api_keys.var.credentials
        let chain = parse_context_chain("@work@website.api_keys.var.credentials", "home")?;
        
        assert_eq!(chain.base, Some("work".to_string()));
        assert_eq!(chain.project, "website");
        assert_eq!(chain.workspace, "api_keys");
        assert_eq!(chain.anchor, Anchor::Var);
        assert_eq!(chain.tail, "credentials");
        assert_eq!(chain.prefix_mode, ChainMode::Persistent);
        assert!(chain.is_fqcc);
        
        // Test CDCC: @frontend.deployment.var.production
        let chain = parse_context_chain("@frontend.deployment.var.production", "work")?;
        
        assert_eq!(chain.base, Some("work".to_string()));
        assert_eq!(chain.project, "frontend");
        assert_eq!(chain.workspace, "deployment");
        assert_eq!(chain.anchor, Anchor::Var);
        assert_eq!(chain.tail, "production");
        assert!(!chain.is_fqcc);
        
        Ok(())
    }
    
    #[test]
    fn test_chain_mode_parsing() -> Result<()> {
        // Persistent mode (@)
        let persistent = parse_context_chain("@proj.workspace.var.keystore", "home")?;
        assert_eq!(persistent.prefix_mode, ChainMode::Persistent);
        
        // Ephemeral mode (%)
        let ephemeral = parse_context_chain("%proj.workspace.var.keystore", "home")?;
        assert_eq!(ephemeral.prefix_mode, ChainMode::Ephemeral);
        
        // Action mode (#)
        let action = parse_context_chain("#proj.workspace.var.keystore", "home")?;
        assert_eq!(action.prefix_mode, ChainMode::Action);
        
        Ok(())
    }
    
    #[test]
    fn test_anchor_case_insensitive() -> Result<()> {
        let var_upper = parse_context_chain("@proj.workspace.VAR.keystore", "home")?;
        let var_lower = parse_context_chain("@proj.workspace.var.keystore", "home")?;
        let var_mixed = parse_context_chain("@proj.workspace.Var.keystore", "home")?;
        
        assert_eq!(var_upper.anchor, Anchor::Var);
        assert_eq!(var_lower.anchor, Anchor::Var);
        assert_eq!(var_mixed.anchor, Anchor::Var);
        
        let doc_upper = parse_context_chain("@proj.workspace.DOC.document", "home")?;
        let doc_lower = parse_context_chain("@proj.workspace.doc.document", "home")?;
        
        assert_eq!(doc_upper.anchor, Anchor::Doc);
        assert_eq!(doc_lower.anchor, Anchor::Doc);
        
        Ok(())
    }
    
    #[test]
    fn test_reserved_namespace_rejection() {
        // Should reject 'var' and 'doc' as namespace names
        assert!(parse_context_chain("@var.workspace.var.keystore", "home").is_err());
        assert!(parse_context_chain("@proj.doc.var.keystore", "home").is_err());
        assert!(parse_context_chain("@proj.workspace.var.var", "home").is_err());
        assert!(parse_context_chain("@proj.workspace.var.doc", "home").is_err());
    }
    
    #[test]
    fn test_invalid_chain_formats() {
        // Missing prefix
        assert!(parse_context_chain("proj.workspace.var.keystore", "home").is_err());
        
        // Wrong number of parts
        assert!(parse_context_chain("@proj.workspace.var", "home").is_err());
        assert!(parse_context_chain("@proj.workspace.var.keystore.extra", "home").is_err());
        
        // Invalid anchor
        assert!(parse_context_chain("@proj.workspace.invalid.keystore", "home").is_err());
        
        // Empty components
        assert!(parse_context_chain("@.workspace.var.keystore", "home").is_err());
        assert!(parse_context_chain("@proj..var.keystore", "home").is_err());
        assert!(parse_context_chain("@proj.workspace.var.", "home").is_err());
    }
    
    #[test]
    fn test_invincible_superchain_creation() {
        let superchain = DefaultResolver::create_invincible_superchain("home");
        let resolver = DefaultResolver::new();
        
        assert_eq!(superchain.project, "ROOT");
        assert_eq!(superchain.workspace, "GLOBAL");
        assert_eq!(superchain.anchor, Anchor::Var);
        assert_eq!(superchain.tail, "MAIN");
        assert!(resolver.is_invincible_superchain(&superchain));
        
        // Verify it displays correctly
        let display = format!("{}", superchain);
        assert_eq!(display, "@home@ROOT.GLOBAL.var.MAIN");
    }
    
    #[test]
    fn test_context_atomicity_enforcement() -> Result<()> {
        let resolver = DefaultResolver::new();
        
        // Test project change resets workspace and tail
        let old_context = parse_context_chain("@proj1.workspace1.var.store1", "work")?;
        let new_context = parse_context_chain("@proj2.workspace1.var.store1", "work")?;
        
        let resolved = resolver.apply_atomicity(&old_context, &new_context);
        assert_eq!(resolved.project, "proj2");
        assert_eq!(resolved.workspace, "GLOBAL"); // Reset to default
        assert_eq!(resolved.tail, "MAIN"); // Reset to default
        
        // Test workspace change resets tail only
        let old_context = parse_context_chain("@proj1.workspace1.var.store1", "work")?;
        let new_context = parse_context_chain("@proj1.workspace2.var.store1", "work")?;
        
        let resolved = resolver.apply_atomicity(&old_context, &new_context);
        assert_eq!(resolved.project, "proj1"); // Same project
        assert_eq!(resolved.workspace, "workspace2"); // New workspace
        assert_eq!(resolved.tail, "MAIN"); // Reset to default
        
        // Test no change when same context
        let old_context = parse_context_chain("@proj1.workspace1.var.store1", "work")?;
        let new_context = parse_context_chain("@proj1.workspace1.var.store2", "work")?;
        
        let resolved = resolver.apply_atomicity(&old_context, &new_context);
        assert_eq!(resolved.project, "proj1");
        assert_eq!(resolved.workspace, "workspace1");
        assert_eq!(resolved.tail, "store2"); // No reset
        
        Ok(())
    }
    
    #[test]
    fn test_cdcc_resolution() -> Result<()> {
        let chain = parse_context_chain("@proj.workspace.var.store", "fallback")?;
        let cursors = CursorState {
            base_cursor: "work".to_string(),
            context_cursor: None,
        };
        
        let resolver = DefaultResolver::new();
        let resolved = resolver.resolve_cdcc(&chain, &cursors);
        
        assert_eq!(resolved.base, "fallback"); // Uses chain base, not cursor
        assert_eq!(resolved.project, "proj");
        assert_eq!(resolved.workspace, "workspace");
        assert_eq!(resolved.tail, "store");
        
        Ok(())
    }
    
    #[test]
    fn test_installation_guard_blocks_usage() {
        let (config, _temp) = create_test_environment();
        let mut guard = InstallationGuard::new(config);
        
        // Should fail when not installed
        let result = guard.require_installation();
        assert!(result.is_err());
        
        if let Err(BookdbError::NotInstalled(_)) = result {
            // Expected error type
        } else {
            panic!("Expected NotInstalled error");
        }
    }
    
    #[test]
    fn test_installation_process_creates_required_structures() -> Result<()> {
        let (config, _temp) = create_test_environment();
        let mut manager = InstallationManager::new(config.clone());
        
        // Perform installation
        manager.perform_installation()?;
        
        // Verify database file exists
        let home_db_path = config.get_base_path("home");
        assert!(home_db_path.exists());
        
        // Verify installation guard now passes
        let mut guard = InstallationGuard::new(config);
        assert!(guard.require_installation().is_ok());
        
        Ok(())
    }
    
    #[test]
    fn test_installation_creates_invincible_superchain() -> Result<()> {
        let (config, _temp) = create_test_environment();
        let mut manager = InstallationManager::new(config.clone());
        
        // Perform installation
        manager.perform_installation()?;
        
        // Open database and verify superchain exists
        let home_db_path = config.get_base_path("home");
        let database = bookdb::db::Database::open(&home_db_path)?;
        
        let superchain = DefaultResolver::create_invincible_superchain("home");
        let resolver = DefaultResolver::new().resolve_cdcc(&superchain, &CursorState::default());
        
        // Verify invincible marker exists
        let marker = database.get_variable("_INVINCIBLE", &resolver)?;
        assert_eq!(marker, Some("1".to_string()));
        
        Ok(())
    }
    
    #[test]
    fn test_cursor_state_persistence() -> Result<()> {
        let (config, _temp) = create_test_environment();
        let mut manager = ContextManager::new(config);
        
        // Create and save cursor state
        let mut cursor_state = CursorState::default();
        cursor_state.base_cursor = "test_base".to_string();
        
        let test_context = parse_context_chain("@proj.workspace.var.keystore", "home")?;
        cursor_state.context_cursor = Some(test_context.clone());
        
        manager.save_cursor_state(&cursor_state)?;
        
        // Load and verify
        let loaded_state = manager.load_cursor_state()?;
        assert_eq!(loaded_state.base_cursor, "test_base");
        assert_eq!(loaded_state.context_cursor.unwrap().project, "proj");
        
        Ok(())
    }
    
    #[test]
    fn test_context_banner_integration() -> Result<()> {
        let (config, _temp) = create_test_environment();
        let mut manager = ContextManager::new(config);
        
        let context = parse_context_chain("@work@proj.workspace.var.keystore", "home")?;
        
        // This should not panic and should properly format the banner
        manager.show_context_banner(&context)?;
        
        Ok(())
    }
    
    #[test]
    fn test_full_workflow_with_ephemeral_context() -> Result<()> {
        let (config, _temp) = create_test_environment();
        
        // Install BookDB
        let mut installer = InstallationManager::new(config.clone());
        installer.perform_installation()?;
        
        // Test ephemeral context usage
        let ephemeral_context = parse_context_chain("%temp@quick.test.var.check", "home")?;
        assert_eq!(ephemeral_context.prefix_mode, ChainMode::Ephemeral);
        assert_eq!(ephemeral_context.base, Some("temp".to_string()));
        
        Ok(())
    }
    
    #[test]
    fn test_error_handling_and_user_feedback() -> Result<()> {
        // Test that various error conditions produce appropriate user feedback
        
        // Invalid context chain
        let result = parse_context_chain("invalid", "home");
        assert!(result.is_err());
        
        // Missing installation
        let (config, _temp) = create_test_environment();
        let mut guard = InstallationGuard::new(config);
        let result = guard.require_installation();
        assert!(result.is_err());
        
        Ok(())
    }
    
    #[test]
    fn test_display_formatting() -> Result<()> {
        // Test that context chains display correctly for user interfaces
        
        let fqcc = parse_context_chain("@work@website.api_keys.var.credentials", "home")?;
        let display = format!("{}", fqcc);
        assert_eq!(display, "@work@website.api_keys.var.credentials");
        
        let ephemeral = parse_context_chain("%temp.test.doc.readme", "home")?;
        let ephemeral_display = format!("{}", ephemeral);
        assert_eq!(ephemeral_display, "%temp.test.doc.readme");
        
        let action = parse_context_chain("#quick.action.var.test", "home")?;
        let action_display = format!("{}", action);
        assert_eq!(action_display, "#quick.action.var.test");
        
        Ok(())
    }
    
    /// Integration test simulating real user workflow
    #[test]
    fn test_complete_user_workflow() -> Result<()> {
        let (config, _temp) = create_test_environment();
        
        // Step 1: Installation
        let mut installer = InstallationManager::new(config.clone());
        installer.perform_installation()?;
        
        // Step 2: Verify installation guard now passes
        let mut guard = InstallationGuard::new(config.clone());
        guard.require_installation()?;
        
        // Step 3: Context management
        let mut context_manager = ContextManager::new(config.clone());
        let mut cursor_state = context_manager.load_cursor_state()?;
        
        // Step 4: Set a context
        let new_context = parse_context_chain("@myproject.config.var.settings", "home")?;
        context_manager.update_cursor(&new_context, &mut cursor_state)?;
        
        // Step 5: Verify context was saved and can be reloaded
        let reloaded_state = context_manager.load_cursor_state()?;
        assert_eq!(reloaded_state.context_cursor.unwrap().project, "myproject");
        
        Ok(())
    }
}

/// Benchmarks for performance verification
#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;
    
    #[test]
    fn bench_context_parsing() {
        let start = Instant::now();
        
        for _ in 0..1000 {
            let _ = parse_context_chain("@work@website.api_keys.var.credentials", "home");
        }
        
        let duration = start.elapsed();
        println!("1000 context parses took: {:?}", duration);
        
        // Should be very fast - context parsing is critical path
        assert!(duration.as_millis() < 100, "Context parsing too slow: {:?}", duration);
    }
    
    #[test]
    fn bench_atomicity_resolution() {
        let old_context = parse_context_chain("@proj1.workspace1.var.store1", "work").unwrap();
        let new_context = parse_context_chain("@proj2.workspace1.var.store1", "work").unwrap();
        let resolver = DefaultResolver::new();
        
        let start = Instant::now();
        
        for _ in 0..1000 {
            let _ = resolver.apply_atomicity(&old_context, &new_context);
        }
        
        let duration = start.elapsed();
        println!("1000 atomicity resolutions took: {:?}", duration);
        
        assert!(duration.as_millis() < 50, "Atomicity resolution too slow: {:?}", duration);
    }
}
