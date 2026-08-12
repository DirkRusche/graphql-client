use std::path::PathBuf;

use crate::{generate_module_token_stream_from_string, CodegenMode, GraphQLClientCodegenOptions};

const KEYWORDS_QUERY: &str = include_str!("keywords_query.graphql");
const KEYWORDS_SCHEMA_PATH: &str = "keywords_schema.graphql";

const FOOBARS_QUERY: &str = include_str!("foobars_query.graphql");
const FOOBARS_SCHEMA_PATH: &str = "foobars_schema.graphql";

const SHARED_FRAGMENTS_QUERY: &str = include_str!("shared_fragments_query.graphql");
const SHARED_FRAGMENTS_SCHEMA_PATH: &str = "shared_fragments_schema.graphql";

const TRIMMED_QUERIES_QUERY: &str = include_str!("trimmed_queries_query.graphql");

fn build_schema_path(path: &str) -> PathBuf {
    std::env::current_dir()
        .unwrap()
        .join("src/tests")
        .join(path)
}

#[test]
fn schema_with_keywords_works() {
    let query_string = KEYWORDS_QUERY;
    let schema_path = build_schema_path(KEYWORDS_SCHEMA_PATH);

    let options = GraphQLClientCodegenOptions::new(CodegenMode::Cli);

    let generated_tokens =
        generate_module_token_stream_from_string(query_string, &schema_path, options)
            .expect("Generate keywords module");

    let generated_code = generated_tokens.to_string();

    // Parse generated code. All keywords should be correctly escaped.
    let r: syn::parse::Result<proc_macro2::TokenStream> = syn::parse2(generated_tokens);
    match r {
        Ok(_) => {
            // Rust keywords should be escaped / renamed now
            assert!(generated_code.contains("pub in_"));
            assert!(generated_code.contains("extern_"));
        }
        Err(e) => {
            panic!("Error: {}\n Generated content: {}\n", e, &generated_code);
        }
    };
}

#[test]
fn blended_custom_types_works() {
    let query_string = KEYWORDS_QUERY;
    let schema_path = build_schema_path(KEYWORDS_SCHEMA_PATH);

    let mut options = GraphQLClientCodegenOptions::new(CodegenMode::Cli);
    options.set_custom_response_type("external_crate::Transaction".to_string());
    options.set_custom_variable_types(vec!["external_crate::ID".to_string()]);

    let generated_tokens =
        generate_module_token_stream_from_string(query_string, &schema_path, options)
            .expect("Generate keywords module");

    let generated_code = generated_tokens.to_string();

    // Parse generated code. Variables and returns should be replaced with custom types
    let r: syn::parse::Result<proc_macro2::TokenStream> = syn::parse2(generated_tokens);
    match r {
        Ok(_) => {
            // Variables and returns should be replaced with custom types
            assert!(generated_code
                .contains("pub type SearchQuerySearch = external_crate :: Transaction"));
            assert!(generated_code.contains("pub type extern_ = external_crate :: ID"));
        }
        Err(e) => {
            panic!("Error: {}\n Generated content: {}\n", e, &generated_code);
        }
    };
}

#[test]
fn fragments_other_variant_should_generate_unknown_other_variant() {
    let query_string = FOOBARS_QUERY;
    let schema_path = build_schema_path(FOOBARS_SCHEMA_PATH);

    let mut options = GraphQLClientCodegenOptions::new(CodegenMode::Cli);

    options.set_fragments_other_variant(true);

    let generated_tokens =
        generate_module_token_stream_from_string(query_string, &schema_path, options)
            .expect("Generate foobars module");

    let generated_code = generated_tokens.to_string();

    let r: syn::parse::Result<proc_macro2::TokenStream> = syn::parse2(generated_tokens);
    match r {
        Ok(_) => {
            // Rust keywords should be escaped / renamed now
            assert!(generated_code.contains("# [serde (other)] Unknown"));
            assert!(generated_code.contains("Unknown"));
        }
        Err(e) => {
            panic!("Error: {}\n Generated content: {}\n", e, &generated_code);
        }
    };
}

#[test]
fn fragments_other_variant_false_should_not_generate_unknown_other_variant() {
    let query_string = FOOBARS_QUERY;
    let schema_path = build_schema_path(FOOBARS_SCHEMA_PATH);

    let mut options = GraphQLClientCodegenOptions::new(CodegenMode::Cli);

    options.set_fragments_other_variant(false);

    let generated_tokens =
        generate_module_token_stream_from_string(query_string, &schema_path, options)
            .expect("Generate foobars module token stream");

    let generated_code = generated_tokens.to_string();

    let r: syn::parse::Result<proc_macro2::TokenStream> = syn::parse2(generated_tokens);
    match r {
        Ok(_) => {
            // Rust keywords should be escaped / renamed now
            assert!(!generated_code.contains("# [serde (other)] Unknown"));
            assert!(!generated_code.contains("Unknown"));
        }
        Err(e) => {
            panic!("Error: {}\n Generated content: {}\n", e, &generated_code);
        }
    };
}

#[test]
fn skip_serializing_none_should_generate_serde_skip_serializing() {
    let query_string = KEYWORDS_QUERY;
    let schema_path = build_schema_path(KEYWORDS_SCHEMA_PATH);

    let mut options = GraphQLClientCodegenOptions::new(CodegenMode::Cli);

    options.set_skip_serializing_none(true);

    let generated_tokens =
        generate_module_token_stream_from_string(query_string, &schema_path, options)
            .expect("Generate foobars module");

    let generated_code = generated_tokens.to_string();

    let r: syn::parse::Result<proc_macro2::TokenStream> = syn::parse2(generated_tokens);

    match r {
        Ok(_) => {
            println!("{}", generated_code);
            assert!(generated_code.contains("skip_serializing_if"));
        }
        Err(e) => {
            panic!("Error: {}\n Generated content: {}\n", e, &generated_code);
        }
    };
}

#[test]
fn shared_fragments_generates_shared_module() {
    let query_string = SHARED_FRAGMENTS_QUERY;
    let schema_path = build_schema_path(SHARED_FRAGMENTS_SCHEMA_PATH);

    let options = GraphQLClientCodegenOptions::new(CodegenMode::Cli);

    let generated_tokens =
        generate_module_token_stream_from_string(query_string, &schema_path, options)
            .expect("Generate shared fragments module");

    let generated_code = generated_tokens.to_string();

    // Parse generated code to verify it's valid Rust.
    let r: syn::parse::Result<proc_macro2::TokenStream> = syn::parse2(generated_tokens);
    match r {
        Ok(_) => {
            // The __shared module should exist
            assert!(
                generated_code.contains("mod __shared"),
                "Expected __shared module in generated code:\n{}",
                generated_code
            );

            // The UserFields fragment struct should appear exactly once (in __shared)
            let user_fields_count = generated_code.matches("struct UserFields").count();
            assert_eq!(
                user_fields_count, 1,
                "Expected UserFields struct to appear exactly once, found {} times in:\n{}",
                user_fields_count, generated_code
            );

            // The Status enum should appear exactly once (in __shared)
            let status_count = generated_code.matches("enum Status").count();
            assert_eq!(
                status_count, 1,
                "Expected Status enum to appear exactly once, found {} times in:\n{}",
                status_count, generated_code
            );

            // Operation modules should re-export from __shared
            assert!(
                generated_code.contains("pub use super :: __shared :: *"),
                "Expected pub use super::__shared::* in generated code:\n{}",
                generated_code
            );
        }
        Err(e) => {
            panic!("Error: {}\n Generated content: {}\n", e, &generated_code);
        }
    };
}

#[test]
fn shared_fragments_not_generated_in_derive_mode() {
    let query_string = SHARED_FRAGMENTS_QUERY;
    let schema_path = build_schema_path(SHARED_FRAGMENTS_SCHEMA_PATH);

    let mut options = GraphQLClientCodegenOptions::new(CodegenMode::Derive);
    options.set_struct_name("GetUser".to_string());
    options.set_operation_name("GetUser".to_string());

    let generated_tokens =
        generate_module_token_stream_from_string(query_string, &schema_path, options)
            .expect("Generate derive mode module");

    let generated_code = generated_tokens.to_string();

    // Parse generated code to verify it's valid Rust.
    let r: syn::parse::Result<proc_macro2::TokenStream> = syn::parse2(generated_tokens);
    match r {
        Ok(_) => {
            // No __shared module in derive mode
            assert!(
                !generated_code.contains("mod __shared"),
                "Expected no __shared module in derive mode:\n{}",
                generated_code
            );

            // Fragment should be defined inline in the operation module
            assert!(
                generated_code.contains("UserFields"),
                "Expected UserFields in derive mode output:\n{}",
                generated_code
            );
        }
        Err(e) => {
            panic!("Error: {}\n Generated content: {}\n", e, &generated_code);
        }
    };
}

/// Extract the value of each generated module's `QUERY` constant, keyed by module name.
fn extract_module_query_strings(file: &syn::File) -> std::collections::BTreeMap<String, String> {
    let mut queries = std::collections::BTreeMap::new();
    for item in &file.items {
        if let syn::Item::Mod(module) = item {
            let items = match &module.content {
                Some((_, items)) => items,
                None => continue,
            };
            for item in items {
                if let syn::Item::Const(constant) = item {
                    if constant.ident == "QUERY" {
                        if let syn::Expr::Lit(syn::ExprLit {
                            lit: syn::Lit::Str(lit_str),
                            ..
                        }) = &*constant.expr
                        {
                            queries.insert(module.ident.to_string(), lit_str.value());
                        }
                    }
                }
            }
        }
    }
    queries
}

#[test]
fn query_strings_are_trimmed_per_operation() {
    let schema_path = build_schema_path(SHARED_FRAGMENTS_SCHEMA_PATH);
    let options = GraphQLClientCodegenOptions::new(CodegenMode::Cli);

    let generated_tokens =
        generate_module_token_stream_from_string(TRIMMED_QUERIES_QUERY, &schema_path, options)
            .expect("Generate trimmed queries module");

    let file: syn::File =
        syn::parse2(generated_tokens).expect("Generated code must be a valid Rust file");
    let queries = extract_module_query_strings(&file);

    let details_query = queries
        .get("get_user_details")
        .expect("get_user_details module has a QUERY constant");
    let list_query = queries
        .get("list_user_names")
        .expect("list_user_names module has a QUERY constant");

    // Each operation's QUERY contains its own operation and not the other one.
    assert!(details_query.contains("GetUserDetails"));
    assert!(
        !details_query.contains("ListUserNames"),
        "GetUserDetails QUERY must not contain the other operation:\n{}",
        details_query
    );
    assert!(list_query.contains("ListUserNames"));
    assert!(
        !list_query.contains("GetUserDetails"),
        "ListUserNames QUERY must not contain the other operation:\n{}",
        list_query
    );

    // The fragment-using operation keeps both the direct and the transitive fragment.
    assert!(
        details_query.contains("fragment UserBase"),
        "GetUserDetails QUERY must contain the directly used fragment:\n{}",
        details_query
    );
    assert!(
        details_query.contains("fragment ContactInfo"),
        "GetUserDetails QUERY must contain the transitively used fragment:\n{}",
        details_query
    );

    // The other operation contains no fragments at all.
    assert!(
        !list_query.contains("fragment"),
        "ListUserNames QUERY must not contain any fragment:\n{}",
        list_query
    );

    // The trimmed strings must round-trip through the GraphQL parser.
    graphql_parser::parse_query::<&str>(details_query)
        .expect("GetUserDetails QUERY parses standalone");
    graphql_parser::parse_query::<&str>(list_query).expect("ListUserNames QUERY parses standalone");
}

#[test]
fn query_strings_are_trimmed_in_derive_mode() {
    let schema_path = build_schema_path(SHARED_FRAGMENTS_SCHEMA_PATH);

    let mut options = GraphQLClientCodegenOptions::new(CodegenMode::Derive);
    options.set_struct_name("GetUserDetails".to_string());
    options.set_operation_name("GetUserDetails".to_string());

    let generated_tokens =
        generate_module_token_stream_from_string(TRIMMED_QUERIES_QUERY, &schema_path, options)
            .expect("Generate derive mode module");

    let file: syn::File =
        syn::parse2(generated_tokens).expect("Generated code must be a valid Rust file");
    let queries = extract_module_query_strings(&file);

    let details_query = queries
        .get("get_user_details")
        .expect("get_user_details module has a QUERY constant");

    assert!(details_query.contains("GetUserDetails"));
    assert!(!details_query.contains("ListUserNames"));
    assert!(details_query.contains("fragment UserBase"));
    assert!(details_query.contains("fragment ContactInfo"));
    graphql_parser::parse_query::<&str>(details_query)
        .expect("GetUserDetails QUERY parses standalone");
}

#[test]
fn no_shared_module_when_no_fragments() {
    // Use foobars query which has no fragments
    let query_string = FOOBARS_QUERY;
    let schema_path = build_schema_path(FOOBARS_SCHEMA_PATH);

    let options = GraphQLClientCodegenOptions::new(CodegenMode::Cli);

    let generated_tokens =
        generate_module_token_stream_from_string(query_string, &schema_path, options)
            .expect("Generate foobars module");

    let generated_code = generated_tokens.to_string();

    // No __shared module when there are no fragments
    assert!(
        !generated_code.contains("mod __shared"),
        "Expected no __shared module when there are no fragments:\n{}",
        generated_code
    );
}
