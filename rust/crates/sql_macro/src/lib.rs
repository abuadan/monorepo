use proc_macro::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::{Expr, Ident, Token, parse_macro_input};

#[proc_macro]
pub fn keyword(input: TokenStream) -> TokenStream {
    let kw = parse_macro_input!(input as Ident);
    let kw_str = kw.to_string();

    let expanded = quote! {
        chumsky::prelude::select! {
            sql_core::lexer::Token::Word(word) if word.keyword == sql_core::keywords::Keyword::Known(#kw_str) => ()
        }
        .map_with(|_, extra| extra.span())
        .labelled(#kw_str)
    };

    TokenStream::from(expanded)
}

#[proc_macro]
pub fn punct(input: TokenStream) -> TokenStream {
    let punct_ident = parse_macro_input!(input as Ident);

    let expanded = quote! {
        sql_core::helpers::punct(sql_core::lexer::Token::#punct_ident)
    };

    TokenStream::from(expanded)
}

#[proc_macro]
pub fn seq(input: TokenStream) -> TokenStream {
    let exprs = parse_macro_input!(input with Punctuated::<Expr, Token![,]>::parse_terminated);

    let mut iter = exprs.into_iter();
    let first = match iter.next() {
        Some(expr) => expr,
        None => return TokenStream::from(quote! { chumsky::prelude::empty() }),
    };

    let mut chained = quote! { #first };
    let mut tuple_structure = quote! { a };
    let mut idents = vec![quote! { a }];

    let mut current_char = b'b';

    for expr in iter {
        chained = quote! { #chained.then(#expr) };

        let ident_name = String::from_utf8(vec![current_char]).unwrap();
        let ident = Ident::new(&ident_name, proc_macro2::Span::call_site());
        let ident_quote = quote! { #ident };

        tuple_structure = quote! { (#tuple_structure, #ident_quote) };
        idents.push(ident_quote);

        current_char += 1;
        if current_char > b'z' {
            panic!("Too many arguments in seq! (max 26)");
        }
    }

    if idents.len() > 1 {
        let return_tuple = quote! { (#(#idents),*) };
        let expanded = quote! {
            #chained.map(|#tuple_structure| #return_tuple)
        };
        TokenStream::from(expanded)
    } else {
        TokenStream::from(chained)
    }
}
