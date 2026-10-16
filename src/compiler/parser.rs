use crate::error::{OrcrustratorError, Result};
use crate::oop::RelationshipDeclaration;
use std::fs;
use std::path::Path;
use syn::visit::Visit;
use syn::{ItemMacro, Macro, Token};

/// Visitor that searches Rust AST syntax trees for `relation!` macro declarations
pub struct RelationVisitor {
    pub declarations: Vec<RelationshipDeclaration>,
}

impl RelationVisitor {
    pub fn new() -> Self {
        Self {
            declarations: Vec::new(),
        }
    }
}

impl<'ast> Visit<'ast> for RelationVisitor {
    fn visit_item_macro(&mut self, node: &'ast ItemMacro) {
        if is_relation_macro(&node.mac) {
            match parse_relation_macro(&node.mac) {
                Ok(decl) => self.declarations.push(decl),
                Err(e) => eprintln!("Compiler warning: failed to parse relation! item macro: {}", e),
            }
        }
        syn::visit::visit_item_macro(self, node);
    }

    fn visit_expr_macro(&mut self, node: &'ast syn::ExprMacro) {
        if is_relation_macro(&node.mac) {
            match parse_relation_macro(&node.mac) {
                Ok(decl) => self.declarations.push(decl),
                Err(e) => eprintln!("Compiler warning: failed to parse relation! expr macro: {}", e),
            }
        }
        syn::visit::visit_expr_macro(self, node);
    }

    fn visit_stmt_macro(&mut self, node: &'ast syn::StmtMacro) {
        if is_relation_macro(&node.mac) {
            match parse_relation_macro(&node.mac) {
                Ok(decl) => self.declarations.push(decl),
                Err(e) => eprintln!("Compiler warning: failed to parse relation! stmt macro: {}", e),
            }
        }
        syn::visit::visit_stmt_macro(self, node);
    }

    fn visit_macro(&mut self, node: &'ast Macro) {
        if is_relation_macro(node) {
            match parse_relation_macro(node) {
                Ok(decl) => self.declarations.push(decl),
                Err(e) => eprintln!("Compiler warning: failed to parse relation! macro: {}", e),
            }
        }
        syn::visit::visit_macro(self, node);
    }
}

fn is_relation_macro(mac: &Macro) -> bool {
    mac.path.is_ident("relation")
        || mac
            .path
            .segments
            .last()
            .map(|s| s.ident == "relation")
            .unwrap_or(false)
}

/// Parse the token stream inside `relation! { ... }`
fn parse_relation_macro(mac: &Macro) -> Result<RelationshipDeclaration> {
    let tokens = mac.tokens.clone();
    let parser = |input: syn::parse::ParseStream| -> syn::Result<RelationshipDeclaration> {
        let mut id = None;
        let mut source = None;
        let mut target = None;
        let mut relation_type = None;
        let mut foreign_key = None;
        let mut embed_as = None;

        while !input.is_empty() {
            let key: syn::Ident = input.parse()?;
            input.parse::<Token![:]>()?;

            match key.to_string().as_str() {
                "id" => {
                    let lit: syn::LitStr = input.parse()?;
                    id = Some(lit.value());
                }
                "source" => {
                    let lit: syn::LitStr = input.parse()?;
                    source = Some(lit.value());
                }
                "target" => {
                    let lit: syn::LitStr = input.parse()?;
                    target = Some(lit.value());
                }
                "relation_type" => {
                    // Could be an identifier like `HasOne` or string `"HasOne"`
                    if input.peek(syn::LitStr) {
                        let lit: syn::LitStr = input.parse()?;
                        relation_type = Some(lit.value());
                    } else {
                        let ident: syn::Ident = input.parse()?;
                        relation_type = Some(ident.to_string());
                    }
                }
                "foreign_key" => {
                    let lit: syn::LitStr = input.parse()?;
                    foreign_key = Some(lit.value());
                }
                "embed_as" => {
                    let lit: syn::LitStr = input.parse()?;
                    embed_as = Some(lit.value());
                }
                _ => {
                    // Skip unrecognized keys gracefully
                    let _: syn::Expr = input.parse()?;
                }
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        let id = id.ok_or_else(|| input.error("Missing required field 'id' in relation!"))?;
        let source = source.ok_or_else(|| input.error("Missing required field 'source' in relation!"))?;
        let target = target.ok_or_else(|| input.error("Missing required field 'target' in relation!"))?;
        let relation_type = relation_type.unwrap_or_else(|| "HasOne".to_string());
        let foreign_key = foreign_key.unwrap_or_default();
        let embed_as = embed_as.unwrap_or_else(|| "data".to_string());

        Ok(RelationshipDeclaration::new(
            id,
            source,
            target,
            relation_type,
            foreign_key,
            embed_as,
        ))
    };

    syn::parse::Parser::parse2(parser, tokens)
        .map_err(|e| OrcrustratorError::CompilerError(format!("Failed to parse relation! macro: {}", e)))
}

/// Recursively scan a source directory and extract all `relation!` declarations
pub fn scan_source_directory(dir: &Path) -> Result<Vec<RelationshipDeclaration>> {
    let mut declarations = Vec::new();
    let mut visitor = RelationVisitor::new();

    scan_dir_recursive(dir, &mut visitor)?;

    // Deduplicate by relation ID, preserving first found
    let mut seen = std::collections::HashSet::new();
    for decl in visitor.declarations {
        if seen.insert(decl.id.clone()) {
            declarations.push(decl);
        }
    }

    // Sort deterministically by relation ID to prevent arbitrary diffs
    declarations.sort_by(|a, b| a.id.cmp(&b.id));

    Ok(declarations)
}

fn scan_dir_recursive(dir: &Path, visitor: &mut RelationVisitor) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }

    if dir.is_file() {
        if dir.extension().map(|e| e == "rs").unwrap_or(false) {
            let content = fs::read_to_string(dir)
                .map_err(|e| OrcrustratorError::CompilerError(format!("Could not read file {:?}: {}", dir, e)))?;
            if let Ok(syntax_tree) = syn::parse_file(&content) {
                visitor.visit_file(&syntax_tree);
            }
        }
        return Ok(());
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            // Skip target and hidden directories
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name != "target" && !name.starts_with('.') {
                scan_dir_recursive(&path, visitor)?;
            }
        } else if path.extension().map(|e| e == "rs").unwrap_or(false) {
            let content = fs::read_to_string(&path)
                .map_err(|e| OrcrustratorError::CompilerError(format!("Could not read file {:?}: {}", path, e)))?;
            match syn::parse_file(&content) {
                Ok(syntax_tree) => visitor.visit_file(&syntax_tree),
                Err(e) => eprintln!("Compiler warning: syn::parse_file failed on {:?}: {}", path, e),
            }
        }
    }

    Ok(())
}
