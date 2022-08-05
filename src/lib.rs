#![doc = include_str!("../README.md")]

/// Derive `sqlx::FromRow` specific to the given database.
///
/// The original derive macro of `sqlx::FromRow` is database-agnostic but too generic to define
/// custom decoder for specific columns. For example, `sqlx::FromRow` cannot support custom decoder
/// like below.
///
/// ```ignore
/// #[derive(sqlx::FromRow)]
/// struct Row {
///     #[sqlx(decode = "split_x")]
///     x: (i64, i64),
/// }
///
/// fn split_x<'r, R>(index: &'r str, row: &'r R) -> sqlx::Result<(i64, i64)>
/// where
///     R: sqlx::Row,
///     &'r str: sqlx::ColumnIndex<R>,
///     i64: sqlx::Type<R::Database> + sqlx::Decode<'r, R::Database>,
/// {
///     let n: i64 = row.try_get(index)?;
///     Ok((n, n + 2))
/// }
/// ```
///
/// The reason is `sqlx::FromRow` cannot add `i64: sqlx::Type<R::Database> + sqlx::Decode<'r, R::Database>`
/// to the derived implementation since it cannot see `row.try_get()` usage from the struct
/// definition.
///
/// sqlx-with resolves the problem by specifying database.
///
/// # Usage
/// Basic usage is similar to `sqlx::FromRow`.
///
/// ```
/// #[derive(sqlx_with::FromRow)]
/// #[sqlx_with(db = "sqlx::Sqlite")]
/// struct Row {
///     x: i64,
///     y: String,
/// }
/// ```
///
/// You have to specify `db`.
///
/// ```compile_fail
/// #[derive(sqlx_with::FromRow)]
/// struct Row {
///     x: i64,
///     y: String,
/// }
/// ```
///
/// You cannot use sqlx-with to tuple structs. Use the original `sqlx::FromRow`
/// instead.
///
/// ```compile_fail
/// #[derive(sqlx_with::FromRow)]
/// #[sqlx_with(db = "sqlx::Sqlite")]
/// struct Row(i64, String);
/// ```
///
/// # Container attributes
/// ## rename_all
/// Specify column name conversion.
///
/// ```
/// #[derive(sqlx_with::FromRow)]
/// #[sqlx_with(db = "sqlx::Sqlite", rename_all = "camelCase")]
/// struct Row {
///     foo_bar: i64,   // deserialized from column "fooBar"
/// }
/// ```
///
/// # Field attributes
/// ## rename
/// Configure column name explicitly. `rename` takes precedence over `rename_all`.
///
/// ```
/// #[derive(sqlx_with::FromRow)]
/// #[sqlx_with(db = "sqlx::Sqlite")]
/// struct Row {
///     #[sqlx_with(rename = "z")]
///     x: i64, // deserialized from column "z"
///     y: String,  // deserialized from column "y"
/// }
/// ```
///
/// ## default
/// Use `Default::default()` value when the column doesn't exist..
///
/// ```
/// #[derive(sqlx_with::FromRow)]
/// #[sqlx_with(db = "sqlx::Sqlite")]
/// struct Row {
///     #[sqlx_with(default)]
///     x: i64, // i64::default() value is set when column "x" doesn't exist.
///     y: String,
/// }
/// ```
///
/// ## decode
/// Configure custom decode function to specific columns.
///
/// ```
/// #[derive(sqlx_with::FromRow)]
/// #[sqlx_with(db = "sqlx::Sqlite")]
/// struct Row {
///     #[sqlx_with(decode = "split_x")]
///     x: (i64, i64),
///     y: String,
/// }
///
/// fn split_x(index: &str, row: &sqlx::sqlite::SqliteRow) -> sqlx::Result<(i64, i64)> {
///     use sqlx::Row as _;
///     let n: i64 = row.try_get(index)?;
///     Ok((n, n + 2))
/// }
/// ```
#[proc_macro_derive(FromRow, attributes(sqlx_with))]
pub fn derive_sqlx_with(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    match expand_derive(input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[derive(Debug, darling::FromMeta)]
enum RenameAll {
    #[darling(rename = "snake_case")]
    Snake,
    #[darling(rename = "lowercase")]
    Lower,
    #[darling(rename = "UPPERCASE")]
    Upper,
    #[darling(rename = "camelCase")]
    Camel,
    #[darling(rename = "PascalCase")]
    Pascal,
    #[darling(rename = "SCREAMING_SNAKE_CASE")]
    ScreamingSnake,
    #[darling(rename = "kebab-case")]
    Kebab,
}

#[derive(Debug, darling::FromDeriveInput)]
#[darling(attributes(sqlx_with), supports(struct_named))]
struct DeriveInput {
    ident: syn::Ident,
    generics: syn::Generics,
    data: darling::ast::Data<(), Field>,
    db: syn::Path,
    rename_all: Option<RenameAll>,
}

#[derive(Debug, darling::FromField)]
#[darling(attributes(sqlx_with))]
struct Field {
    ident: Option<syn::Ident>,
    rename: Option<String>,
    default: darling::util::Flag,
    decode: Option<syn::Path>,
}

fn expand_derive(input: syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    use darling::FromDeriveInput as _;

    let input = DeriveInput::from_derive_input(&input)?;

    let mut struct_expr: syn::ExprStruct = syn::parse_quote!(Self {});
    for field in input.data.take_struct().unwrap().fields {
        let id = field.ident.unwrap();
        let column_name = if let Some(rename) = field.rename {
            rename
        } else if let Some(ref rename_all) = input.rename_all {
            use heck::*;

            match rename_all {
                RenameAll::Snake => id.to_string().to_snake_case(),
                RenameAll::Lower => id.to_string().to_lowercase(),
                RenameAll::Upper => id.to_string().to_uppercase(),
                RenameAll::Camel => id.to_string().to_lower_camel_case(),
                RenameAll::Pascal => id.to_string().to_upper_camel_case(),
                RenameAll::ScreamingSnake => id.to_string().to_shouty_snake_case(),
                RenameAll::Kebab => id.to_string().to_kebab_case(),
            }
        } else {
            id.to_string()
        };
        let column_get_expr: syn::Expr = if let Some(decode) = field.decode {
            syn::parse_quote!(#decode(#column_name, row))
        } else {
            syn::parse_quote!(row.try_get(#column_name))
        };
        let column_val_expr: syn::Expr = if field.default.is_present() {
            syn::parse_quote! {
                match #column_get_expr {
                    ::std::result::Result::Err(::sqlx::Error::ColumnNotFound(_)) => ::std::result::Result::Ok(::std::default::Default::default()),
                    val => val,
                }?
            }
        } else {
            syn::parse_quote!(#column_get_expr?)
        };
        struct_expr
            .fields
            .push(syn::parse_quote!(#id: #column_val_expr));
    }

    let struct_ident = input.ident;
    let db = input.db;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
    Ok(quote::quote! {
        impl #impl_generics ::sqlx::FromRow<'_, <#db as ::sqlx::Database>::Row> for #struct_ident #type_generics #where_clause {
            fn from_row(row: &<#db as ::sqlx::Database>::Row) -> ::sqlx::Result<Self> {
                use ::sqlx::Row;
                Ok(#struct_expr)
            }
        }
    })
}
