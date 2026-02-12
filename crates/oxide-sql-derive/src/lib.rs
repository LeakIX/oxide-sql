//! Derive macros for type-safe SQL table definitions.
//!
//! This crate provides the `#[derive(Table)]` macro for defining database tables
//! with compile-time checked column names.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{Attribute, Data, DeriveInput, Expr, Fields, Ident, Lit, Meta, Type, parse_macro_input};

/// Derives the `Table` trait for a struct, generating type-safe column accessors.
///
/// # Attributes
///
/// - `#[table(name = "table_name")]` - Specifies the SQL table name (optional,
///   defaults to snake_case of struct name)
///
/// # Field Attributes
///
/// - `#[column(primary_key)]` - Marks the field as primary key
/// - `#[column(name = "column_name")]` - Specifies the SQL column name
///   (optional, defaults to field name)
/// - `#[column(nullable)]` - Marks the column as nullable
/// - `#[column(unique)]` - Marks the column as UNIQUE
/// - `#[column(autoincrement)]` - Marks the column as AUTOINCREMENT
/// - `#[column(default = "expr")]` - Sets a raw SQL default expression
///
/// # Generated Items
///
/// For a struct `User`, this macro generates:
///
/// - `UserTable` - A type implementing `Table` trait with table metadata
/// - `UserColumns` - A module containing column types (`Id`, `Name`, etc.)
/// - Column accessor methods on `UserTable`
#[proc_macro_derive(Table, attributes(table, column))]
pub fn derive_table(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    derive_table_impl(input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

fn derive_table_impl(input: DeriveInput) -> syn::Result<TokenStream2> {
    let struct_name = &input.ident;
    let table_name = get_table_name(&input.attrs, struct_name)?;

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => {
                return Err(syn::Error::new_spanned(
                    &input,
                    "Table derive only supports structs with named fields",
                ));
            }
        },
        _ => {
            return Err(syn::Error::new_spanned(
                &input,
                "Table derive only supports structs",
            ));
        }
    };

    // Collect field information
    let mut column_infos: Vec<ColumnInfo> = Vec::new();
    for field in fields {
        let field_name = field.ident.as_ref().unwrap();
        let field_type = &field.ty;
        let column_attrs = parse_column_attrs(&field.attrs)?;

        column_infos.push(ColumnInfo {
            field_name: field_name.clone(),
            field_type: field_type.clone(),
            column_name: column_attrs.name.unwrap_or_else(|| field_name.to_string()),
            is_primary_key: column_attrs.primary_key,
            is_nullable: column_attrs.nullable,
            is_unique: column_attrs.unique,
            is_autoincrement: column_attrs.autoincrement,
            default_expr: column_attrs.default_expr,
        });
    }

    // Generate column type names (PascalCase)
    let column_type_names: Vec<Ident> = column_infos
        .iter()
        .map(|c| format_ident!("{}", to_pascal_case(&c.field_name.to_string())))
        .collect();

    // Generate the table struct name
    let table_struct_name = format_ident!("{}Table", struct_name);
    let columns_mod_name = format_ident!("{}Columns", struct_name);

    // Generate column structs
    let column_structs: Vec<TokenStream2> = column_infos
        .iter()
        .zip(column_type_names.iter())
        .map(|(info, type_name)| {
            let column_name = &info.column_name;
            let field_type = &info.field_type;
            let is_nullable = info.is_nullable;
            let is_primary_key = info.is_primary_key;

            quote! {
                /// Column type for compile-time checked queries.
                #[derive(Debug, Clone, Copy)]
                pub struct #type_name;

                impl ::oxide_sql_core::schema::Column for #type_name {
                    type Table = super::#table_struct_name;
                    type Type = #field_type;

                    const NAME: &'static str = #column_name;
                    const NULLABLE: bool = #is_nullable;
                    const PRIMARY_KEY: bool = #is_primary_key;
                }

                impl ::oxide_sql_core::schema::TypedColumn<#field_type> for #type_name {}
            }
        })
        .collect();

    // Generate column accessor methods
    let column_accessors: Vec<TokenStream2> = column_infos
        .iter()
        .zip(column_type_names.iter())
        .map(|(info, type_name)| {
            let method_name = &info.field_name;
            quote! {
                /// Returns the column type for type-safe queries.
                #[inline]
                pub const fn #method_name() -> #columns_mod_name::#type_name {
                    #columns_mod_name::#type_name
                }
            }
        })
        .collect();

    // Generate list of all column names
    let all_column_names: Vec<&str> = column_infos
        .iter()
        .map(|c| c.column_name.as_str())
        .collect();

    // Find primary key column
    let primary_key_column = column_infos
        .iter()
        .find(|c| c.is_primary_key)
        .map(|c| &c.column_name);

    let primary_key_impl = if let Some(pk) = primary_key_column {
        quote! {
            const PRIMARY_KEY: Option<&'static str> = Some(#pk);
        }
    } else {
        quote! {
            const PRIMARY_KEY: Option<&'static str> = None;
        }
    };

    // Generate TableSchema column entries
    let schema_entries: Vec<TokenStream2> = column_infos
        .iter()
        .map(|info| {
            let col_name = &info.column_name;
            let field_type = &info.field_type;
            let rust_type_str = quote!(#field_type).to_string().replace(' ', "");
            let is_nullable = info.is_nullable;
            let is_primary_key = info.is_primary_key;
            let is_unique = info.is_unique;
            let is_autoincrement = info.is_autoincrement;
            let default_expr_token = match &info.default_expr {
                Some(expr) => quote! { Some(#expr) },
                None => quote! { None },
            };

            quote! {
                ::oxide_sql_core::schema::ColumnSchema {
                    name: #col_name,
                    rust_type: #rust_type_str,
                    nullable: #is_nullable,
                    primary_key: #is_primary_key,
                    unique: #is_unique,
                    autoincrement: #is_autoincrement,
                    default_expr: #default_expr_token,
                }
            }
        })
        .collect();

    let expanded = quote! {
        /// Column types for `#struct_name` table.
        #[allow(non_snake_case)]
        pub mod #columns_mod_name {
            #(#column_structs)*
        }

        /// Table metadata for `#struct_name`.
        #[derive(Debug, Clone, Copy)]
        pub struct #table_struct_name;

        impl ::oxide_sql_core::schema::Table for #table_struct_name {
            type Row = #struct_name;

            const NAME: &'static str = #table_name;
            const COLUMNS: &'static [&'static str] = &[#(#all_column_names),*];
            #primary_key_impl
        }

        impl ::oxide_sql_core::schema::TableSchema
            for #table_struct_name
        {
            const SCHEMA: &'static [
                ::oxide_sql_core::schema::ColumnSchema
            ] = &[
                #(#schema_entries),*
            ];
        }

        impl #table_struct_name {
            /// Returns the table name.
            #[inline]
            pub const fn table_name() -> &'static str {
                #table_name
            }

            #(#column_accessors)*
        }

        impl #struct_name {
            /// Returns the table metadata type.
            pub fn table() -> #table_struct_name {
                #table_struct_name
            }

            #(#column_accessors)*
        }
    };

    Ok(expanded)
}

struct ColumnInfo {
    field_name: Ident,
    field_type: Type,
    column_name: String,
    is_primary_key: bool,
    is_nullable: bool,
    is_unique: bool,
    is_autoincrement: bool,
    default_expr: Option<String>,
}

struct ColumnAttrs {
    name: Option<String>,
    primary_key: bool,
    nullable: bool,
    unique: bool,
    autoincrement: bool,
    default_expr: Option<String>,
}

fn get_table_name(attrs: &[Attribute], struct_name: &Ident) -> syn::Result<String> {
    for attr in attrs {
        if attr.path().is_ident("table") {
            let mut table_name = None;
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("name") {
                    let value: Expr = meta.value()?.parse()?;
                    if let Expr::Lit(lit) = value {
                        if let Lit::Str(s) = lit.lit {
                            table_name = Some(s.value());
                        }
                    }
                }
                Ok(())
            })?;
            if let Some(name) = table_name {
                return Ok(name);
            }
        }
    }
    // Default to snake_case of struct name
    Ok(to_snake_case(&struct_name.to_string()))
}

fn parse_column_attrs(attrs: &[Attribute]) -> syn::Result<ColumnAttrs> {
    let mut result = ColumnAttrs {
        name: None,
        primary_key: false,
        nullable: false,
        unique: false,
        autoincrement: false,
        default_expr: None,
    };

    for attr in attrs {
        if attr.path().is_ident("column") {
            // Handle empty attribute like #[column]
            if matches!(attr.meta, Meta::Path(_)) {
                continue;
            }

            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("primary_key") {
                    result.primary_key = true;
                } else if meta.path.is_ident("nullable") {
                    result.nullable = true;
                } else if meta.path.is_ident("unique") {
                    result.unique = true;
                } else if meta.path.is_ident("autoincrement") {
                    result.autoincrement = true;
                } else if meta.path.is_ident("name") {
                    let value: Expr = meta.value()?.parse()?;
                    if let Expr::Lit(lit) = value {
                        if let Lit::Str(s) = lit.lit {
                            result.name = Some(s.value());
                        }
                    }
                } else if meta.path.is_ident("default") {
                    let value: Expr = meta.value()?.parse()?;
                    if let Expr::Lit(lit) = value {
                        if let Lit::Str(s) = lit.lit {
                            result.default_expr = Some(s.value());
                        }
                    }
                }
                Ok(())
            })?;
        }
    }

    Ok(result)
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }
    result
}

fn to_pascal_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;
    for c in s.chars() {
        if c == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }
    result
}
