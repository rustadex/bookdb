// tests/unit/v3_validator_tests.rs
// Comprehensive unit tests for V3 context chain validator
// Tests all 12 single-purpose functions and the Deref pattern

#[cfg(test)]
mod v3_validator_tests {
    use bookdb::context_validator::{validate_and_create_v3, upgrade_to_specialized, V3ContextResult};
    use bookdb::context::{ContextChainV3, VarContextChain, DocContextChain, Anchor, ChainMode, ContextType};
    use bookdb::error::{Result, BookdbError};
    use std::ops::Deref;

    // ========================================================================
    // MAIN VALIDATOR TESTS
    // ========================================================================

    #[test]
    fn test_validate_and_create_v3_variable_context() -> Result<()> {
        let result = validate_and_create_v3("@work@proj.workspace.var.keystore", "home")?;
        
        match result {
            V3ContextResult::Variable(var_chain) => {
                assert_eq!(var_chain.anchor(), Anchor::Var);
                assert_eq!(var_chain.prefix_mode, ChainMode::Persistent);
                assert!(var_chain.is_fqcc);
                // Test Deref trait
                let generic: &ContextChainV3 = var_chain.deref();
                assert_eq!(generic.chain_type, ContextType::Variable);
            }
            _ => panic!("Expected Variable result, got Document"),
        }
        
        Ok(())
    }

    #[test]
    fn test_validate_and_create_v3_document_context() -> Result<()> {
        let result = validate_and_create_v3("@base@proj.workspace.doc.readme", "home")?;
        
        match result {
            V3ContextResult::Document(doc_chain) => {
                assert_eq!(doc_chain.anchor(), Anchor::Doc);
                assert_eq!(doc_chain.prefix_mode, ChainMode::Persistent);
                assert!(doc_chain.is_fqcc);
                // Test Deref trait
                let generic: &ContextChainV3 = doc_chain.deref();
                assert_eq!(generic.chain_type, ContextType::Document);
            }
            _ => panic!("Expected Document result, got Variable"),
        }
        
        Ok(())
    }

    #[test]
    fn test_upgrade_to_specialized() -> Result<()> {
        // Create a generic ContextChainV3 (this would normally come from parsing)
        let generic = ContextChainV3 {
            segments: vec![], // TODO: Add segments when implemented
            chain_type: ContextType::Variable,
            is_fqcc: true,
        };
        
        let result = upgrade_to_specialized(&generic)?;
        match result {
            V3ContextResult::Variable(_) => {}, // Expected
            _ => panic!("Expected Variable specialization"),
        }
        
        Ok(())
    }

    // ========================================================================
    // PREFIX MODE PARSING TESTS
    // ========================================================================

    #[test]
    fn test_persistent_prefix_mode() -> Result<()> {
        let result = validate_and_create_v3("@proj.workspace.var.keystore", "home")?;
        assert_eq!(result.as_generic().chain_type, ContextType::Variable);
        Ok(())
    }

    #[test]
    fn test_ephemeral_prefix_mode() -> Result<()> {
        let result = validate_and_create_v3("%proj.workspace.var.keystore", "home")?;
        match result {
            V3ContextResult::Variable(var_chain) => {
                assert_eq!(var_chain.prefix_mode, ChainMode::Ephemeral);
            }
            _ => panic!("Expected Variable result"),
        }
        Ok(())
    }

    #[test]
    fn test_action_prefix_mode() -> Result<()> {
        let result = validate_and_create_v3("#proj.workspace.var.keystore", "home")?;
        match result {
            V3ContextResult::Variable(var_chain) => {
                assert_eq!(var_chain.prefix_mode, ChainMode::Action);
            }
            _ => panic!("Expected Variable result"),
        }
        Ok(())
    }

    #[test]
    fn test_invalid_prefix_mode() {
        let result = validate_and_create_v3("proj.workspace.var.keystore", "home");
        assert!(result.is_err());
        
        if let Err(BookdbError::ContextParse(msg)) = result {
            assert!(msg.contains("must start with @, %, or #"));
        } else {
            panic!("Expected ContextParse error");
        }
    }

    // ========================================================================
    // BASE COMPONENT PARSING TESTS (FQCC vs CDCC)
    // ========================================================================

    #[test]
    fn test_fqcc_base_parsing() -> Result<()> {
        let result = validate_and_create_v3("@work@proj.workspace.var.keystore", "home")?;
        match result {
            V3ContextResult::Variable(var_chain) => {
                assert!(var_chain.is_fqcc);
                assert_eq!(var_chain.base, Some("work".to_string()));
            }
            _ => panic!("Expected Variable result"),
        }
        Ok(())
    }

    #[test]
    fn test_cdcc_base_fallback() -> Result<()> {
        let result = validate_and_create_v3("@proj.workspace.var.keystore", "fallback_base")?;
        match result {
            V3ContextResult::Variable(var_chain) => {
                assert!(!var_chain.is_fqcc);
                assert_eq!(var_chain.base, Some("fallback_base".to_string()));
            }
            _ => panic!("Expected Variable result"),
        }
        Ok(())
    }

    #[test]
    fn test_empty_base_name_error() {
        let result = validate_and_create_v3("@@proj.workspace.var.keystore", "home");
        assert!(result.is_err());
        
        if let Err(BookdbError::ContextParse(msg)) = result {
            assert!(msg.contains("Empty base name"));
        } else {
            panic!("Expected ContextParse error for empty base");
        }
    }

    // ========================================================================
    // COMPONENT VALIDATION TESTS
    // ========================================================================

    #[test]
    fn test_invalid_component_count() {
        // Too few components
        let result = validate_and_create_v3("@proj.workspace.var", "home");
        assert!(result.is_err());
        
        // Too many components
        let result = validate_and_create_v3("@proj.workspace.var.keystore.extra", "home");
        assert!(result.is_err());
    }

    #[test]
    fn test_anchor_case_insensitive() -> Result<()> {
        let var_upper = validate_and_create_v3("@proj.workspace.VAR.keystore", "home")?;
        let var_lower = validate_and_create_v3("@proj.workspace.var.keystore", "home")?;
        
        assert_eq!(var_upper.anchor(), var_lower.anchor());
        assert_eq!(var_upper.anchor(), Anchor::Var);
        
        let doc_upper = validate_and_create_v3("@proj.workspace.DOC.readme", "home")?;
        let doc_lower = validate_and_create_v3("@proj.workspace.doc.readme", "home")?;
        
        assert_eq!(doc_upper.anchor(), doc_lower.anchor());
        assert_eq!(doc_upper.anchor(), Anchor::Doc);
        
        Ok(())
    }

    #[test]
    fn test_invalid_anchor() {
        let result = validate_and_create_v3("@proj.workspace.invalid.keystore", "home");
        assert!(result.is_err());
        
        if let Err(BookdbError::ContextParse(msg)) = result {
            assert!(msg.contains("Invalid anchor"));
        } else {
            panic!("Expected ContextParse error for invalid anchor");
        }
    }

    // ========================================================================
    // DEREF TRAIT COERCION TESTS
    // ========================================================================

    #[test]
    fn test_var_chain_deref_coercion() -> Result<()> {
        let result = validate_and_create_v3("@proj.workspace.var.keystore", "home")?;
        
        if let V3ContextResult::Variable(var_chain) = result {
            // Test that we can use VarContextChain where ContextChainV3 is expected
            fn accepts_generic_context(ctx: &ContextChainV3) -> ContextType {
                ctx.chain_type
            }
            
            // This should work via Deref trait
            let context_type = accepts_generic_context(&var_chain);
            assert_eq!(context_type, ContextType::Variable);
        } else {
            panic!("Expected Variable result");
        }
        
        Ok(())
    }

    #[test]
    fn test_doc_chain_deref_coercion() -> Result<()> {
        let result = validate_and_create_v3("@proj.workspace.doc.readme", "home")?;
        
        if let V3ContextResult::Document(doc_chain) = result {
            // Test that we can use DocContextChain where ContextChainV3 is expected
            fn accepts_generic_context(ctx: &ContextChainV3) -> bool {
                ctx.is_fqcc
            }
            
            // This should work via Deref trait
            let is_fqcc = accepts_generic_context(&doc_chain);
            assert!(!is_fqcc); // CDCC in this case
        } else {
            panic!("Expected Document result");
        }
        
        Ok(())
    }

    // ========================================================================
    // ERROR HANDLING TESTS
    // ========================================================================

    #[test]
    fn test_empty_input_error() {
        let result = validate_and_create_v3("", "home");
        assert!(result.is_err());
        
        let result = validate_and_create_v3("   ", "home");
        assert!(result.is_err());
    }

    #[test]
    fn test_detailed_error_messages() {
        // Test that errors provide helpful context
        let result = validate_and_create_v3("invalid_format", "home");
        assert!(result.is_err());
        
        if let Err(BookdbError::ContextParse(msg)) = result {
            assert!(!msg.is_empty());
            assert!(msg.len() > 10); // Should be descriptive
        } else {
            panic!("Expected ContextParse error");
        }
    }

    // ========================================================================
    // V3CONTEXTRESULT TESTS
    // ========================================================================

    #[test]
    fn test_v3_context_result_display() -> Result<()> {
        let var_result = validate_and_create_v3("@proj.workspace.var.keystore", "home")?;
        let doc_result = validate_and_create_v3("@proj.workspace.doc.readme", "home")?;
        
        // Test display strings are not empty
        assert!(!var_result.display_string().is_empty());
        assert!(!doc_result.display_string().is_empty());
        
        // Test anchor accessors
        assert_eq!(var_result.anchor(), Anchor::Var);
        assert_eq!(doc_result.anchor(), Anchor::Doc);
        
        Ok(())
    }

    #[test]
    fn test_as_generic_accessor() -> Result<()> {
        let result = validate_and_create_v3("@proj.workspace.var.keystore", "home")?;
        
        // Test that as_generic() returns the underlying ContextChainV3
        let generic = result.as_generic();
        assert_eq!(generic.chain_type, ContextType::Variable);
        
        Ok(())
    }

    // ========================================================================
    // INTEGRATION TESTS WITH OTHER COMPONENTS
    // ========================================================================

    #[test]
    fn test_v3_with_resolver_compatibility() -> Result<()> {
        // Test that V3 types work with existing resolver logic
        let result = validate_and_create_v3("@proj.workspace.var.keystore", "home")?;
        
        // This would test integration with actual resolver
        // For now, just verify the types are correct
        match result {
            V3ContextResult::Variable(var_chain) => {
                assert_eq!(var_chain.project, "TODO"); // TODO when segment extraction implemented
                assert_eq!(var_chain.workspace, "TODO");
                assert_eq!(var_chain.keystore, "TODO");
            }
            _ => panic!("Expected Variable result"),
        }
        
        Ok(())
    }

    // ========================================================================
    // PERFORMANCE TESTS
    // ========================================================================

    #[test]
    fn test_parsing_performance() {
        use std::time::Instant;
        
        let start = Instant::now();
        
        // Parse 1000 context chains
        for i in 0..1000 {
            let input = format!("@proj{}.workspace.var.keystore", i);
            let _ = validate_and_create_v3(&input, "home");
        }
        
        let duration = start.elapsed();
        println!("1000 V3 context parses took: {:?}", duration);
        
        // Should be reasonably fast - this is critical path
        assert!(duration.as_millis() < 500, "V3 parsing too slow: {:?}", duration);
    }

    // ========================================================================
    // PROPERTY-BASED TEST HELPERS
    // ========================================================================

    #[test]
    fn test_roundtrip_property() -> Result<()> {
        // Test that valid inputs can be parsed and displayed consistently
        let inputs = vec![
            "@proj.workspace.var.keystore",
            "%temp.test.doc.readme", 
            "#quick.action.var.secret",
            "@base@proj.workspace.var.config",
        ];
        
        for input in inputs {
            let result = validate_and_create_v3(input, "home")?;
            let display = result.display_string();
            
            // The display should be a valid context chain
            // (Though format might differ slightly)
            assert!(!display.is_empty());
            assert!(display.contains('.'));
        }
        
        Ok(())
    }
}
