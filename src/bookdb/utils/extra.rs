


use stderr::{Stderr, StderrConfig, BorderStyle};

/// Table formatter for ls commands with stderr integration
pub struct LsTableFormatter {
    logger: Stderr,
}

impl LsTableFormatter {
    pub fn new() -> Self {
        Self {
            logger: Stderr::new(&StderrConfig::from_env()),
        }
    }
    
    /// Format and display a table of items
    pub fn display_table(&mut self, headers: &[&str], rows: &[Vec<String>], title: Option<&str>) -> Result<()> {
        if let Some(title) = title {
            self.logger.banner(title, '=')?;
        }
        
        if rows.is_empty() {
            self.logger.info("No items found.");
            return Ok(());
        }
        
        // Convert to table format
        let mut table_data = vec![headers.to_vec()];
        for row in rows {
            table_data.push(row.iter().map(|s| s.as_str()).collect());
        }
        
        // Convert to &[&[&str]] format required by simple_table
        let table_refs: Vec<Vec<&str>> = table_data.iter()
            .map(|row| row.iter().map(|s| s.as_str()).collect())
            .collect();
        let table_slice: Vec<&[&str]> = table_refs.iter()
            .map(|row| row.as_slice())
            .collect();
        
        self.logger.simple_table(&table_slice)?;
        
        self.logger.info(&format!("Total: {} items", rows.len()));
        
        Ok(())
    }
    
    /// Display variables in a nice table format
    pub fn display_variables(&mut self, variables: &[(String, String)], context: &str) -> Result<()> {
        let title = format!("Variables in {}", context);
        
        let rows: Vec<Vec<String>> = variables.iter()
            .map(|(key, value)| vec![key.clone(), value.clone()])
            .collect();
        
        self.display_table(&["Key", "Value"], &rows, Some(&title))
    }
    
    /// Display projects, workspaces, etc. in table format
    pub fn display_namespaces(&mut self, items: &[String], namespace_type: &str, context: &str) -> Result<()> {
        let title = format!("{} in {}", namespace_type, context);
        
        let rows: Vec<Vec<String>> = items.iter()
            .enumerate()
            .map(|(i, name)| vec![(i + 1).to_string(), name.clone()])
            .collect();
        
        self.display_table(&["#", "Name"], &rows, Some(&title))
    }
}



/// Progress tracking for long operations
pub struct OperationProgress {
    logger: Stderr,
    operation_name: String,
    total_items: Option<usize>,
    current_item: usize,
}

impl OperationProgress {
    pub fn new(operation_name: &str) -> Self {
        let logger = Stderr::new(&StderrConfig::from_env());
        Self {
            logger,
            operation_name: operation_name.to_string(),
            total_items: None,
            current_item: 0,
        }
    }
    
    pub fn set_total(&mut self, total: usize) {
        self.total_items = Some(total);
        self.logger.trace_fn("progress", &format!("{}: starting {} items", self.operation_name, total));
    }
    
    pub fn increment(&mut self, item_name: &str) -> Result<()> {
        self.current_item += 1;
        
        if let Some(total) = self.total_items {
            let percent = (self.current_item as f64 / total as f64 * 100.0) as usize;
            self.logger.info(&format!("[{}/{}] ({:3}%) Processing: {}", 
                self.current_item, total, percent, item_name));
        } else {
            self.logger.info(&format!("[{}] Processing: {}", self.current_item, item_name));
        }
        
        Ok(())
    }
    
    pub fn complete(&mut self) -> Result<()> {
        self.logger.okay(&format!("{} completed successfully! Processed {} items.", 
            self.operation_name, self.current_item));
        Ok(())
    }
}



/// Confirmation helper for destructive operations
pub struct DestructiveOpConfirm {
    logger: Stderr,
}

impl DestructiveOpConfirm {
    pub fn new() -> Self {
        Self {
            logger: Stderr::new(&StderrConfig::from_env()),
        }
    }
    
    /// Confirm deletion operations
    pub fn confirm_delete(&mut self, item_type: &str, item_name: &str) -> Result<bool> {
        self.logger.warn(&format!("About to delete {} '{}'", item_type, item_name));
        self.logger.error("This operation cannot be undone!");
        
        let result = self.logger.confirm_builder(&format!("Delete {} '{}'?", item_type, item_name))
            .boxed(true)
            .style(BorderStyle::Heavy)
            .ask()?
            .unwrap_or(false);
        
        if result {
            self.logger.trace_fn("destructive_op", &format!("user confirmed deletion of {}", item_name));
        } else {
            self.logger.trace_fn("destructive_op", "user cancelled deletion");
        }
        
        Ok(result)
    }
    
    /// Confirm import/export operations that might overwrite data
    pub fn confirm_overwrite(&mut self, operation: &str, target: &str) -> Result<bool> {
        self.logger.warn(&format!("About to {} - this may overwrite existing data", operation));
        self.logger.info(&format!("Target: {}", target));
        
        let result = self.logger.confirm(&format!("Continue with {}?", operation))?
            .unwrap_or(false);
        
        if result {
            self.logger.trace_fn("destructive_op", &format!("user confirmed {}", operation));
        } else {
            self.logger.trace_fn("destructive_op", &format!("user cancelled {}", operation));
        }
        
        Ok(result)
    }
    
    /// Confirm reset operations
    pub fn confirm_reset(&mut self, scope: &str) -> Result<bool> {
        self.logger.error(&format!("About to reset {} - this will delete all data in scope!", scope));
        self.logger.error("This operation cannot be undone!");
        
        // Require typing "yes" for reset operations
        self.logger.info("");
        self.logger.info("Type 'yes' to confirm reset:");
        
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let confirmed = input.trim().to_lowercase() == "yes";
        
        if confirmed {
            self.logger.trace_fn("destructive_op", &format!("user confirmed reset of {}", scope));
            self.logger.warn("Reset confirmed. Proceeding...");
        } else {
            self.logger.trace_fn("destructive_op", "user cancelled reset");
            self.logger.info("Reset cancelled.");
        }
        
        Ok(confirmed)
    }
}



