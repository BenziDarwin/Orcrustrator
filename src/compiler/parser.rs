use crate::error::{OrcrustratorError, Result};
use crate::oop::RelationshipDeclaration;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use syn::visit::Visit;
use syn::{ItemMacro, ItemStruct, Macro, Token};

/// Represents an inferred or explicitly declared field on a struct
#[derive(Debug, Clone)]
struct FieldInfo {
    name: String,
    vec_inner_ty: Option<String>,
    relation_attr: Option<RelationFieldAttr>,
}

#[derive(Debug, Clone, Default)]
struct RelationFieldAttr {
    relation_type: Option<String>,
    target: Option<String>,
    source: Option<String>,
    foreign_key: Option<String>,
    embed_as: Option<String>,
    id: Option<String>,
}

#[derive(Debug, Clone)]
struct StructInfo {
    name: String,
    fields: Vec<FieldInfo>,
    custom_source: Option<String>,
}

/// Visitor that searches Rust AST syntax trees for:
/// 1. `relation!` macro declarations
/// 2. Struct definitions with field relations (Vec<T>, *_id, *_code, #[relation(...)])
/// 3. REST endpoint route string literals (/api/v1/...)
/// 4. Relational builder and stitcher method calls
pub struct RelationVisitor {
    pub declarations: Vec<RelationshipDeclaration>,
    discovered_structs: Vec<StructInfo>,
    discovered_routes: Vec<String>,
    discovered_builders: Vec<RelationshipDeclaration>,
}

impl RelationVisitor {
    pub fn new() -> Self {
        Self {
            declarations: Vec::new(),
            discovered_structs: Vec::new(),
            discovered_routes: Vec::new(),
            discovered_builders: Vec::new(),
        }
    }

    /// Post-process discovered structs, routes, and builder calls to infer relationships
    pub fn infer_relations_from_code(&mut self) {
        // 1. Add builder declarations
        for decl in &self.discovered_builders {
            if !self.declarations.iter().any(|d| d.id == decl.id) {
                self.declarations.push(decl.clone());
            }
        }

        // 2. Infer relations from route path hierarchies
        let mut sorted_routes = self.discovered_routes.clone();
        sorted_routes.sort_by_key(|r| r.len());

        let mut route_parent_map: HashMap<String, String> = HashMap::new();

        for child_route in &sorted_routes {
            // Check if route matches hierarchical pattern: /prefix/{param}/suffix
            if let Some(open_idx) = child_route.find("/{") {
                let parent_prefix = &child_route[..open_idx];
                if let Some(matching_parent) = sorted_routes.iter().find(|r| *r == parent_prefix) {
                    let after_open = &child_route[open_idx + 2..];
                    if let Some(close_idx) = after_open.find('}') {
                        let param_name = &after_open[..close_idx];
                        let rest = &after_open[close_idx + 1..];
                        let sub_resource = rest.trim_start_matches('/');

                        if !sub_resource.is_empty() {
                            let parent_name = parent_prefix.split('/').filter(|s| !s.is_empty()).last().unwrap_or("parent");
                            let rel_id = format!("{}_{}", parent_name, sub_resource.replace('/', "_"));

                            if !self.declarations.iter().any(|d| d.id == rel_id) {
                                self.declarations.push(RelationshipDeclaration::new(
                                    rel_id,
                                    matching_parent.clone(),
                                    child_route.clone(),
                                    "HasMany",
                                    param_name,
                                    sub_resource.replace('/', "_"),
                                ));
                            }

                            route_parent_map.insert(child_route.clone(), matching_parent.clone());
                        }
                    }
                }
            }
        }

        // 3. Infer relations from struct models
        for s in &self.discovered_structs {
            let snake_name = pascal_to_snake(&s.name);
            let default_source = s.custom_source.clone().unwrap_or_else(|| {
                // Find matching route in discovered routes or generate default
                let plural = pluralize(&snake_name);
                self.discovered_routes
                    .iter()
                    .find(|r| r.ends_with(&format!("/{}", plural)) || r.ends_with(&format!("/{}", snake_name)))
                    .cloned()
                    .unwrap_or_else(|| format!("/api/v1/{}", plural))
            });

            for f in &s.fields {
                // Check if field has explicit #[relation(...)] attribute
                if let Some(attr) = &f.relation_attr {
                    let rel_type = attr.relation_type.clone().unwrap_or_else(|| "HasOne".to_string());
                    let embed_as = attr.embed_as.clone().unwrap_or_else(|| f.name.clone());
                    let fk = attr.foreign_key.clone().unwrap_or_else(|| f.name.clone());
                    let src = attr.source.clone().unwrap_or_else(|| default_source.clone());
                    let tgt = attr.target.clone().unwrap_or_else(|| format!("{}/{{{}}}", default_source, fk));
                    let rel_id = attr.id.clone().unwrap_or_else(|| format!("{}_{}", snake_name, f.name));

                    if !self.declarations.iter().any(|d| d.id == rel_id) {
                        self.declarations.push(RelationshipDeclaration::new(
                            rel_id,
                            src,
                            tgt,
                            rel_type,
                            fk,
                            embed_as,
                        ));
                    }
                    continue;
                }

                // Automatic inference: Vec<ChildType> -> HasMany
                if let Some(inner_ty) = &f.vec_inner_ty {
                    let child_snake = pascal_to_snake(inner_ty);
                    let rel_id = format!("{}_{}", snake_name, f.name);

                    // Look for child endpoint in discovered routes
                    let target_endpoint = self.discovered_routes.iter().find(|r| {
                        r.starts_with(&default_source) && (r.ends_with(&f.name) || r.ends_with(&child_snake))
                    }).cloned().unwrap_or_else(|| {
                        format!("{}/{{{}_id}}/{}", default_source, snake_name, f.name)
                    });

                    // Infer foreign key: check if param is present in target endpoint
                    let foreign_key = if let Some(open) = target_endpoint.find("/{") {
                        if let Some(close) = target_endpoint[open + 2..].find('}') {
                            target_endpoint[open + 2..open + 2 + close].to_string()
                        } else {
                            format!("{}_id", snake_name)
                        }
                    } else {
                        format!("{}_id", snake_name)
                    };

                    if !self.declarations.iter().any(|d| d.id == rel_id) {
                        self.declarations.push(RelationshipDeclaration::new(
                            rel_id,
                            default_source.clone(),
                            target_endpoint,
                            "HasMany",
                            foreign_key,
                            f.name.clone(),
                        ));
                    }
                }
                // Automatic inference: *_id or *_code -> HasOne
                else if let Some((entity, _)) = extract_fk_entity(&f.name) {
                    if entity != snake_name {
                        let rel_id = format!("{}_{}", snake_name, entity);
                        let entity_plural = pluralize(entity);

                        // Look for matching route for target entity
                        let target_endpoint = self.discovered_routes.iter().find(|r| {
                            r.contains(&format!("/{}", entity_plural)) || r.contains(&format!("/{}", entity))
                        }).cloned().unwrap_or_else(|| {
                            format!("/api/v1/{}/{{{}}}", entity_plural, f.name)
                        });

                        if !self.declarations.iter().any(|d| d.id == rel_id) {
                            self.declarations.push(RelationshipDeclaration::new(
                                rel_id,
                                default_source.clone(),
                                target_endpoint,
                                "HasOne",
                                f.name.clone(),
                                entity.to_string(),
                            ));
                        }
                    }
                }
            }
        }

        // 4. Ensure root nodes exist for distinct API route sources (e.g. /api/v1/...)
        let non_roots: Vec<RelationshipDeclaration> = self.declarations.iter().filter(|d| d.relation_type != "Root").cloned().collect();
        let mut distinct_sources: HashSet<String> = HashSet::new();

        for d in &non_roots {
            if d.source.starts_with("/api/") || d.source.starts_with("/v1/") {
                distinct_sources.insert(d.source.clone());
            }
        }

        for src in distinct_sources {
            let already_has_root = self.declarations.iter().any(|d| d.relation_type == "Root" && (d.source == src || d.target == src));
            if !already_has_root {
                let clean_name = src.split('/').filter(|s| !s.is_empty() && *s != "api" && !s.starts_with('v')).last().unwrap_or("root");
                let root_id = format!("{}_root", clean_name);

                if !self.declarations.iter().any(|d| d.id == root_id) {
                    let fk = format!("{}_code", clean_name);
                    self.declarations.insert(0, RelationshipDeclaration::new(
                        root_id,
                        src.clone(),
                        src.clone(),
                        "Root",
                        fk,
                        "root",
                    ));
                }
            }
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

    fn visit_item_struct(&mut self, node: &'ast ItemStruct) {
        let struct_name = node.ident.to_string();
        let mut custom_source = None;

        for attr in &node.attrs {
            if attr.path().is_ident("relation") {
                let _ = attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("source") {
                        let value = meta.value()?;
                        let s: syn::LitStr = value.parse()?;
                        custom_source = Some(s.value());
                    }
                    Ok(())
                });
            }
        }

        let mut fields = Vec::new();
        if let syn::Fields::Named(named_fields) = &node.fields {
            for field in &named_fields.named {
                let field_name = field.ident.as_ref().map(|i| i.to_string()).unwrap_or_default();
                let vec_inner = extract_vec_inner_type(&field.ty);

                let mut rel_attr = None;
                for attr in &field.attrs {
                    if attr.path().is_ident("relation") {
                        let mut parsed_attr = RelationFieldAttr::default();
                        let _ = attr.parse_nested_meta(|meta| {
                            if meta.path.is_ident("has_many") {
                                parsed_attr.relation_type = Some("HasMany".to_string());
                            } else if meta.path.is_ident("has_one") {
                                parsed_attr.relation_type = Some("HasOne".to_string());
                            } else if meta.path.is_ident("target") {
                                let val = meta.value()?;
                                let s: syn::LitStr = val.parse()?;
                                parsed_attr.target = Some(s.value());
                            } else if meta.path.is_ident("source") {
                                let val = meta.value()?;
                                let s: syn::LitStr = val.parse()?;
                                parsed_attr.source = Some(s.value());
                            } else if meta.path.is_ident("fk") || meta.path.is_ident("foreign_key") {
                                let val = meta.value()?;
                                let s: syn::LitStr = val.parse()?;
                                parsed_attr.foreign_key = Some(s.value());
                            } else if meta.path.is_ident("embed_as") {
                                let val = meta.value()?;
                                let s: syn::LitStr = val.parse()?;
                                parsed_attr.embed_as = Some(s.value());
                            } else if meta.path.is_ident("id") {
                                let val = meta.value()?;
                                let s: syn::LitStr = val.parse()?;
                                parsed_attr.id = Some(s.value());
                            }
                            Ok(())
                        });
                        rel_attr = Some(parsed_attr);
                    }
                }

                fields.push(FieldInfo {
                    name: field_name,
                    vec_inner_ty: vec_inner,
                    relation_attr: rel_attr,
                });
            }
        }

        self.discovered_structs.push(StructInfo {
            name: struct_name,
            fields,
            custom_source,
        });

        syn::visit::visit_item_struct(self, node);
    }

    fn visit_lit_str(&mut self, node: &'ast syn::LitStr) {
        let val = node.value();
        if (val.starts_with("/api/") || val.starts_with("/v1/") || (val.starts_with('/') && val.contains("/{") && val.matches('/').count() >= 2))
            && !val.contains(' ') && !val.contains('\n')
        {
            if !self.discovered_routes.contains(&val) {
                self.discovered_routes.push(val);
            }
        }
        syn::visit::visit_lit_str(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        let method_name = node.method.to_string();
        if method_name == "has_many" || method_name == "has_one" {
            if let Some(decl) = extract_builder_method_decl(node) {
                self.discovered_builders.push(decl);
            }
        }
        syn::visit::visit_expr_method_call(self, node);
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

fn extract_builder_method_decl(node: &syn::ExprMethodCall) -> Option<RelationshipDeclaration> {
    let method_name = node.method.to_string();
    let rel_type = if method_name == "has_many" { "HasMany" } else { "HasOne" };

    let mut str_args = Vec::new();
    for arg in &node.args {
        if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) = arg {
            str_args.push(s.value());
        }
    }

    if !str_args.is_empty() {
        let name = str_args.first()?.clone();
        let target = str_args.get(1).cloned().unwrap_or_else(|| format!("/{}", name));
        let fk = str_args.get(2).cloned().unwrap_or_else(|| "id".to_string());
        let embed_as = str_args.last().cloned().unwrap_or_else(|| name.clone());

        return Some(RelationshipDeclaration::new(
            format!("builder_{}", name),
            "/api/v1/root",
            target,
            rel_type,
            fk,
            embed_as,
        ));
    }
    None
}

fn extract_vec_inner_type(ty: &syn::Type) -> Option<String> {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            if segment.ident == "Vec" {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                        if let syn::Type::Path(inner_path) = inner_ty {
                            return inner_path.path.segments.last().map(|s| s.ident.to_string());
                        }
                    }
                }
            } else if segment.ident == "Option" {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                        return extract_vec_inner_type(inner_ty);
                    }
                }
            }
        }
    }
    None
}

fn extract_fk_entity(field_name: &str) -> Option<(&str, &str)> {
    if let Some(entity) = field_name.strip_suffix("_id") {
        if !entity.is_empty() {
            return Some((entity, "id"));
        }
    }
    if let Some(entity) = field_name.strip_suffix("_code") {
        if !entity.is_empty() {
            return Some((entity, "code"));
        }
    }
    None
}

fn pascal_to_snake(s: &str) -> String {
    let mut result = String::new();
    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(ch.to_ascii_lowercase());
        } else {
            result.push(ch);
        }
    }
    result
}

fn pluralize(s: &str) -> String {
    if s.ends_with('y') && !s.ends_with("ay") && !s.ends_with("ey") && !s.ends_with("oy") && !s.ends_with("uy") {
        format!("{}ies", &s[..s.len() - 1])
    } else if s.ends_with('s') || s.ends_with("ch") || s.ends_with("sh") || s.ends_with('x') || s.ends_with('z') {
        format!("{}es", s)
    } else {
        format!("{}s", s)
    }
}

/// Recursively scan a source directory and extract all relation declarations and inferred relations
pub fn scan_source_directory(dir: &Path) -> Result<Vec<RelationshipDeclaration>> {
    let mut visitor = RelationVisitor::new();

    scan_dir_recursive(dir, &mut visitor)?;

    // Infer relations from structs, routes, and builder patterns
    visitor.infer_relations_from_code();

    // Deduplicate by relation ID, preserving explicit macros first
    let mut declarations = Vec::new();
    let mut seen = HashSet::new();
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
