use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream}, Expr, LitStr,
    Token,
};
use syn::{parse_macro_input, Lit};

enum Cardinality {
    One,
    Optional,
    All,
}

impl Cardinality {
    fn parse(ident: &syn::Ident) -> syn::Result<Self> {
        match ident.to_string().as_str() {
            "ONE" => Ok(Cardinality::One),
            "OPTIONAL" => Ok(Cardinality::Optional),
            "ALL" => Ok(Cardinality::All),
            _ => Err(syn::Error::new(
                ident.span(),
                "Cardinality must be ONE | OPTIONAL | ALL",
            )),
        }
    }
}

enum Shape {
    Row,
    Column,
}

impl Shape {
    fn parse(ident: &syn::Ident) -> syn::Result<Self> {
        match ident.to_string().as_str() {
            "ROW" => Ok(Shape::Row),
            "COLUMN" => Ok(Shape::Column),
            _ => Err(syn::Error::new(ident.span(), "Shape must be ROW | COLUMN")),
        }
    }
}

struct DbCallInput {
    pool: Expr,
    query: LitStr,
    binds: Vec<Expr>,
    cardinality: Cardinality,
    shape: Shape,
}

impl Parse for DbCallInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut pool = None;
        let mut query = None;
        let mut binds = Vec::new();
        let mut cardinality = None;
        let mut shape = None;

        while !input.is_empty() {
            let ident: syn::Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            match ident.to_string().as_str() {
                "pool" => pool = Some(input.parse()?),
                "query" => {
                    if !input.peek(syn::Ident) {
                        return Err(syn::Error::new(
                            ident.span(),
                            "expected `ONE`, `OPTIONAL`, or `ALL` after `fetch =`",
                        ));
                    }

                    let next_ident: syn::Ident = input.parse()?;
                    cardinality = Some(Cardinality::parse(&next_ident)?);

                    let next_ident: syn::Ident = input.parse()?;
                    shape = Some(Shape::parse(&next_ident)?);

                    if let Expr::Lit(expr) = input.parse()?
                        && let Lit::Str(lit_str) = expr.lit
                    {
                        query = Some(lit_str)
                    } else {
                        return Err(syn::Error::new(
                            ident.span(),
                            "Query must be a literal string",
                        ));
                    }
                }
                "binds" => {
                    let content;
                    syn::bracketed!(content in input);
                    while !content.is_empty() {
                        binds.push(content.parse()?);
                        let _ = content.parse::<Token![,]>();
                    }
                }
                _ => return Err(syn::Error::new(ident.span(), "Unknown argument")),
            }

            let _ = input.parse::<Token![,]>();
        }

        Ok(Self {
            pool: pool.ok_or_else(|| input.error("Missing pool"))?,
            query: query.ok_or_else(|| input.error("Missing sql"))?,
            binds,
            cardinality: cardinality.ok_or_else(|| input.error("Missing cardinality"))?,
            shape: shape.ok_or_else(|| input.error("Missing shape"))?,
        })
    }
}

#[proc_macro]
pub fn db_call(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DbCallInput);

    let pool = &input.pool;
    let query = &input.query;
    let binds = &input.binds;
    let cardinality = &input.cardinality;
    let shape = &input.shape;

    let fetch_shape = match shape {
        Shape::Row => quote! { query_as },
        Shape::Column => quote! { query_scalar },
    };

    let fetch_cardinality = match cardinality {
        Cardinality::One => quote! { fetch_one },
        Cardinality::Optional => quote! { fetch_optional },
        Cardinality::All => quote! { fetch_all },
    };

    let map_err = quote! {
        |err: sqlx::Error| {
                if let sqlx::Error::Database(db_err) = &err
                    && let Some(code) = db_err.code().as_deref()
                {
                    let mapped =
                        <_ as easy_errors::DbErrorTrait>::from_code(code);

                    if <_ as easy_errors::DbErrorTrait>::is_unexpected(&mapped) {
                        easy_errors::log::error!(
                            "UNEXPECTED SQLx ERROR (unmapped CODE {code}): {err:?}"
                        );
                    }

                    return mapped;
                }

                easy_errors::log::error!("UNEXPECTED SQLx ERROR: {err:?}");
                <_ as ::easy_errors::DbErrorTrait>::unexpected(err)
            }
    };

    quote! {{
        let mut query = sqlx::#fetch_shape(#query);
        #( query = query.bind(#binds); )*

        query.#fetch_cardinality(#pool)
            .await
            .map_err(#map_err)
    }}
    .into()
}
