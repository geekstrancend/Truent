//! Invariant DSL grammar definition using pest.
#![allow(missing_docs)]

use pest_derive::Parser;

/// The Truent DSL grammar.
#[allow(non_camel_case_types)]
#[derive(Parser)]
#[grammar_inline = r#"
WHITESPACE = _{ " " | "\t" | NEWLINE }
NEWLINE = @{ "\r\n" | "\n" }

// Identifiers and literals
identifier = @{ (ASCII_ALPHA | "_") ~ (ASCII_ALPHANUMERIC | "_")* }
layer_name = @{ "bundler" | "account" | "paymaster" | "protocol" | "entrypoint" }
integer = @{ "-"? ~ ASCII_DIGIT+ }

// Operators (ordered by precedence)
eq = { "==" }
neq = { "!=" }
lt = { "<" }
gt = { ">" }
lte = { "<=" }
gte = { ">=" }
and = { "&&" }
or = { "||" }
not = { "!" }

// Literals
boolean = @{ "true" | "false" }

// Qualified identifiers with optional layer scope (layer::identifier)
qualified_id = { layer_name ~ "::" ~ identifier }
simple_id = { identifier }
var_id = { qualified_id | simple_id }

// Function call - must be tried before identifier
function_call = { identifier ~ "(" ~ (expr ~ ("," ~ expr)*)? ~ ")" }

// Atoms: function calls, literals, or identifiers (in order of specificity)
atom = _{ function_call | boolean | integer | var_id }

// Primary expressions with parentheses
primary = { "(" ~ expr ~ ")" | atom }

// Unary operators
unary = { not* ~ primary }

// Comparison operators
comparison = { unary ~ ((eq | neq | lte | gte | lt | gt) ~ unary)* }

// Logical AND
logical_and = { comparison ~ (and ~ comparison)* }

// Logical OR  
logical_or = { logical_and ~ (or ~ logical_and)* }

// Main expression
expr = { logical_or }

// Top-level invariant
invariant_def = {
    "invariant" ~ identifier ~ ("(" ~ layer_name ~ ("," ~ layer_name)* ~ ")")? ~ "{" ~ expr ~ "}"
}

file = { SOI ~ invariant_def+ ~ EOI }
"#]
pub struct TruentGrammar;

pub use TruentGrammar as Grammar;
