// src/main.rs

use anyhow::{Context, Result};
// No longer need: use quote::quote;
use std::env;
use std::fs;
use syn::{ImplItem, Item, TraitItem};

fn main() -> Result<()> {
    // --- Setup: Read arguments and file content ---
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: rustfile_summarizer <path-to-rust-file>");
        std::process::exit(1);
    }

    let file_path = &args[1];
    let code = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read file: {}", file_path))?;

    // Parse the Rust file into an Abstract Syntax Tree (AST)
    let mut ast = syn::parse_file(&code)
        .with_context(|| format!("Failed to parse file: {}", file_path))?;

    // --- Step 1 & 2 combined: Filter and Modify Declarations ---
    // (This part of the code remains exactly the same)
    ast.items.retain_mut(|item| {
        match item {
            Item::Struct(_) => true,
            Item::Enum(_) => true,
            Item::Type(_) => true,
            Item::Const(_) => true,
            Item::Fn(item_fn) => {
                item_fn.block.stmts.clear();
                true
            },
            Item::Trait(item_trait) => {
                for inner_item in &mut item_trait.items {
                    if let TraitItem::Fn(trait_fn) = inner_item {
                        trait_fn.default = None;
                    }
                }
                true
            },
            // Keep impl blocks, but summarize their contents.
            Item::Impl(item_impl) => {
                // Iterate over the items within the impl block (functions, types, consts)
                for inner_item in &mut item_impl.items {
                    // If the item is a function...
                    if let ImplItem::Fn(impl_fn) = inner_item {
                        // ...clear its body statements.
                        impl_fn.block.stmts.clear();
                    }
                    // We automatically keep other items like `type Output = bool;`
                    // or `const ID: u32 = 1;` as they don't have bodies to strip.
                }
                true // Keep the modified impl block
            },
            _ => false,
        }
    });

    // --- Step 3: Print the modified AST with prettyplease ---
    // `prettyplease::unparse` takes a reference to the `syn::File` AST
    // and returns a beautifully formatted String.
    let summarized_code = prettyplease::unparse(&ast);

    println!("{}", summarized_code);

    Ok(())
}
