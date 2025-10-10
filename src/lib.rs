use proc_macro::TokenStream;
use syn::{parse_macro_input};
use quote::{quote, ToTokens};
use proc_macro2::TokenStream as TokenStream2;

fn capitalize_first_letter(s: &str) -> String {
    let mut s = s
        .to_string()
        .into_bytes()
        .into_iter()
        .collect::<Vec<_>>();
    s[0] = s[0].to_ascii_uppercase();
    String::from_utf8(s).unwrap()
}

#[proc_macro]
pub fn rpc_functions(item: TokenStream) -> TokenStream {
    let file = parse_macro_input!(item as syn::File);
    let mut function_names: Vec<syn::Ident> = vec![];
    let mut function_fields: Vec<Vec<syn::Ident>> = vec![];
    let mut function_types: Vec<Vec<syn::Type>> = vec![];
    for item in &file.items {
        match item {
            syn::Item::Fn(item_fn) => {
                if !item_fn.attrs.iter().map(|attr| match attr.meta.clone() {
                    syn::Meta::Path(path) => {
                        path.segments.last()
                            .map(|ps| ps.ident.to_string().eq("rpc"))
                            .unwrap_or(false)
                    },
                    _ => false
                }).any(|b| b) {
                    panic!();
                }
                function_names.push(item_fn.sig.ident.clone());
                function_fields.push(
                    item_fn.sig.inputs.iter().map(|fn_arg| match fn_arg {
                    syn::FnArg::Typed(pat_type) => {match *pat_type.pat.clone() {
                        syn::Pat::Ident(pat_ident) => {pat_ident.ident},
                        _ => panic!()
                    }},
                    _ => panic!()
                }).collect());
                function_types.push(
                    item_fn.sig.inputs.iter().map(|fn_arg| match fn_arg {
                    syn::FnArg::Typed(pat_type) => {
                        *pat_type.ty.clone()
                    },
                    _ => panic!("All functions in must have `#[rpc]' annotation")
                }).collect());
            },
            _ => panic!()
        };
    }

    let enum_variant_names: Vec<proc_macro2::Ident> = function_names
        .iter()
        .map(|s| syn::Ident::new(
            &s.to_string().split("_")
                .map(|s| capitalize_first_letter(s)).collect::<String>(),
            proc_macro2::Span::call_site()
        ))
        .collect();

    let fields = enum_variant_names
        .iter()
        .zip(&function_types)
        .map(|(ev, function_types)| {
            quote!{ #ev(#(#function_types),*) }
        })
        .collect::<Vec<_>>();

    let rpc_enum_struct = quote! {
        #[derive(Serialize, Deserialize)]
        enum RpcArgs {
            #(#fields,)*
        }
    };

    let append_token_streams = |mut b: TokenStream2, i: &syn::Item| { 
        b.extend(i.to_token_stream());
        b
    };

    let functions = file.items.iter().fold(
        TokenStream2::new(), 
        append_token_streams,
    );

    let rpc_call = quote!{
        macro_rules! rpc_call {
            ( $args: expr ) => {
                match $args { #(
                    RpcArgs::#enum_variant_names(
                            #( #function_fields ),*
                        ) => #function_names(#( #function_fields ),*)
                ),* }
            }
        }
    };

    let rpc_defer = quote! {
        macro_rules! rpc_defer {
            #(
                ( #function_names ( $($args:expr),* ) ) => {
                    RpcArgs::#enum_variant_names(
                        $($args),*
                    )
                }
            );*
        }
    };

    let output = quote! {
        #functions
        #rpc_enum_struct
        #rpc_call
        #rpc_defer
    };

    /*
    println!("{}", prettyplease::unparse(
        &syn::parse2::<syn::File>(output.clone()).expect("")
    ));
    */

    return TokenStream::from(output);
}

/// The RPC macro doesn't really do anything, it's moreso a 
/// annotation that helps with readability
#[proc_macro_attribute]
pub fn rpc(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
