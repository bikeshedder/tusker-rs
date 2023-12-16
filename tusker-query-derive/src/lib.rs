use darling::FromDeriveInput;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput};

#[derive(FromDeriveInput)]
#[darling(attributes(query), supports(struct_named))]
struct QueryTraitOpts {
    ident: syn::Ident,
    sql: String,
    row: Option<syn::Ident>,
}

#[proc_macro_derive(Query, attributes(query))]
pub fn derive_query(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let opts = QueryTraitOpts::from_derive_input(&ast).expect("Deriving Query failed");
    let Data::Struct(s) = ast.data else {
        unreachable!();
    };
    let name = opts.ident;
    let sql = opts.sql;
    let row = opts
        .row
        .map(|ident| quote! { #ident })
        .unwrap_or(quote! { () });
    let params = s.fields.iter().map(|field| {
        let field_name = field.ident.as_ref().unwrap();
        quote! {
            &self.#field_name
        }
    });
    quote! {
        impl ::tusker_query::Query for #name {
            const SQL: &'static str = include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/db/queries/",
                #sql
            ));
            type Row = #row;
            fn as_params(&self) -> Box<[&(dyn ::tokio_postgres::types::ToSql + Sync)]> {
                Box::new([
                    #( #params ),*
                ])
            }
        }
    }
    .into()
}

#[derive(FromDeriveInput)]
#[darling(supports(struct_named))]
struct FromRowTraitOpts {
    ident: syn::Ident,
}

#[proc_macro_derive(FromRow)]
pub fn derive_from_row(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let opts = FromRowTraitOpts::from_derive_input(&ast).expect("Deriving FromRow failed");
    let Data::Struct(s) = ast.data else {
        unreachable!();
    };
    let name = opts.ident;
    let fields = s.fields.iter().enumerate().map(|(idx, field)| {
        let field_name = &field.ident;
        quote! {
            #field_name: row.get(#idx)
        }
    });
    quote! {
        impl ::tusker_query::FromRow for #name {
            fn from_row(row: ::tokio_postgres::Row) -> Self {
                Self {
                    #( #fields ),*
                }
            }
        }
    }
    .into()
}
