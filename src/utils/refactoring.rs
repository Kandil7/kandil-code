//! Code refactoring utilities
//!
//! Contains functionality for code refactoring with preview/apply workflow

use anyhow::Result;

#[derive(Debug, Clone)]
pub struct RefactorOperation {
    pub file_path: String,
    original_code: String,
    pub refactored_code: String,
    operation_type: String, // e.g., "rename_variable", "extract_function", etc.
    description: String,
}

impl RefactorOperation {
    pub fn original_code(&self) -> &str {
        &self.original_code
    }

    pub fn operation_type(&self) -> &str {
        &self.operation_type
    }

    pub fn description(&self) -> &str {
        &self.description
    }
}

pub struct RefactorEngine {
    operations: Vec<RefactorOperation>,
}

impl RefactorEngine {
    pub fn new() -> Self {
        Self {
            operations: Vec::new(),
        }
    }

    pub fn preview_refactor(
        &mut self,
        file_path: &str,
        refactor_type: &str,
        params: &RefactorParams,
    ) -> Result<String> {
        let original_code = std::fs::read_to_string(file_path)?;

        // Apply the refactoring transformation
        let refactored_code = self.apply_refactor(&original_code, refactor_type, params)?;

        // Store the operation for potential application
        let operation = RefactorOperation {
            file_path: file_path.to_string(),
            original_code: original_code.clone(),
            refactored_code: refactored_code.clone(),
            operation_type: refactor_type.to_string(),
            description: format!("{} operation on {}", refactor_type, file_path),
        };

        self.operations.push(operation);

        Ok(refactored_code)
    }

    fn apply_refactor(
        &self,
        code: &str,
        refactor_type: &str,
        params: &RefactorParams,
    ) -> Result<String> {
        match refactor_type {
            "rename_variable" => self.rename_variable(code, params),
            "extract_function" => self.extract_function(code, params),
            "rename_function" => self.rename_function(code, params),
            _ => Ok(code.to_string()), // For unknown refactor types, return original
        }
    }

    fn rename_variable(&self, code: &str, params: &RefactorParams) -> Result<String> {
        // This is a simplified implementation
        // In a real implementation, we'd use Tree-sitter to parse and rename variables safely
        if let (Some(old_name), Some(new_name)) = (&params.old_name, &params.new_name) {
            Ok(code.replace(old_name, new_name))
        } else {
            Ok(code.to_string())
        }
    }

    fn rename_function(&self, code: &str, params: &RefactorParams) -> Result<String> {
        // This is a simplified implementation
        // In a real implementation, we'd use Tree-sitter to parse and rename functions safely
        if let (Some(old_name), Some(new_name)) = (&params.old_name, &params.new_name) {
            Ok(code.replace(&format!("fn {}(", old_name), &format!("fn {}(", new_name)))
        } else {
            Ok(code.to_string())
        }
    }

    fn extract_function(&self, code: &str, _params: &RefactorParams) -> Result<String> {
        // This is a simplified implementation
        // In a real implementation, we'd use Tree-sitter to extract code into a new function
        Ok(code.to_string())
    }

    pub fn get_pending_operations(&self) -> &Vec<RefactorOperation> {
        &self.operations
    }

    pub fn apply_pending_operations(&mut self) -> Result<()> {
        for operation in &self.operations {
            std::fs::write(&operation.file_path, &operation.refactored_code)?;
        }
        self.operations.clear();
        Ok(())
    }

    pub fn cancel_pending_operations(&mut self) {
        self.operations.clear();
    }

    pub fn get_operation_preview(&self, index: usize) -> Option<&RefactorOperation> {
        self.operations.get(index)
    }
}

#[derive(Debug, Clone)]
pub struct RefactorParams {
    pub old_name: Option<String>,
    pub new_name: Option<String>,
    pub start_line: Option<usize>,
    pub end_line: Option<usize>,
    pub function_name: Option<String>,
    pub visibility: Option<String>,
}

impl RefactorParams {
    pub fn new() -> Self {
        Self {
            old_name: None,
            new_name: None,
            start_line: None,
            end_line: None,
            function_name: None,
            visibility: None,
        }
    }

    pub fn start_line(&self) -> Option<usize> {
        self.start_line
    }

    pub fn end_line(&self) -> Option<usize> {
        self.end_line
    }

    pub fn function_name(&self) -> Option<&str> {
        self.function_name.as_deref()
    }

    pub fn visibility(&self) -> Option<&str> {
        self.visibility.as_deref()
    }

    pub fn with_start_line(mut self, start_line: usize) -> Self {
        self.start_line = Some(start_line);
        self
    }

    pub fn with_end_line(mut self, end_line: usize) -> Self {
        self.end_line = Some(end_line);
        self
    }

    pub fn with_function_name<T: Into<String>>(mut self, function_name: T) -> Self {
        self.function_name = Some(function_name.into());
        self
    }

    pub fn with_visibility<T: Into<String>>(mut self, visibility: T) -> Self {
        self.visibility = Some(visibility.into());
        self
    }
}
