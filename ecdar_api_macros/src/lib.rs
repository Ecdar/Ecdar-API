use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ImplItemFn, Item, ItemFn, ItemImpl, ItemMod};

#[proc_macro_attribute]
pub fn endpoints(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item_mod: ItemMod = parse_macro_input!(item as ItemMod);

    let impl_names: Vec<String> = item_mod
        .clone()
        .content
        .as_ref()
        .map(|(_, items)| {
            items.iter().filter_map(|item| {
                if let syn::Item::Impl(impl_item) = item {
                    Some(
                        impl_item
                            .trait_
                            .as_ref()
                            .map_or_else(
                                || None,
                                |t| Some(t.1.segments.last().unwrap().ident.to_string()),
                            )
                            .unwrap_or_else(|| "".to_string()),
                    )
                } else {
                    None
                }
            })
        })
        .unwrap()
        .collect();

    let names: Vec<String> = impl_names
        .clone()
        .into_iter()
        .filter(|item| !item.is_empty())
        .collect();

    let mut endpoints: Vec<String> = Vec::new();

    let items_impl: Vec<Item> = item_mod.content.unwrap().1;

    let items_impl: Vec<ItemImpl> = items_impl
        .into_iter()
        .filter_map(|item| match item {
            Item::Impl(item_impl) => Some(item_impl),
            _ => None,
        })
        .collect();

    let items_impl: Vec<ItemImpl> = items_impl
        .clone()
        .into_iter()
        .filter(|item| item.trait_.is_some())
        .collect();

    for i in 0..items_impl.len() {
        let existing_functions: Vec<String> = items_impl[i]
            .items
            .iter()
            .filter_map(|item| match item {
                syn::ImplItem::Fn(function) => Some(function.sig.ident.to_string()),
                _ => None,
            })
            .collect();

        for function in existing_functions {
            endpoints.push(format!("{}/{}", names[i], function));
        }
    }

    let new_function: TokenStream = quote! {
        async fn endpoints(&self, request: tonic::Request<()>) -> std::result::Result<tonic::Response<EndpointsResponse>, tonic::Status> {
            let names = vec![#(#endpoints.to_string()),*];
            Ok(Response::new(EndpointsResponse {
                endpoints: names
            }))
        }
    }
    .into();

    println!("{}", new_function.to_string());

    // let mut updated_items = item_mod.content.as_ref()

    // let mut updated_items = input.items.clone();

    // updated_items.push(syn::ImplItem::Fn(parse_macro_input!(
    //     new_function as ImplItemFn
    // )));

    // let updated_impl = ItemImpl {
    //     attrs: input.attrs,
    //     defaultness: input.defaultness,
    //     unsafety: input.unsafety,
    //     impl_token: input.impl_token,
    //     generics: input.generics,
    //     trait_: input.trait_,
    //     self_ty: input.self_ty,
    //     brace_token: input.brace_token,
    //     items: updated_items,
    // };

    // let output = quote! {
    //     #updated_impl
    // };

    // println!("{}", output.to_string());

    // output.into()
    todo!()
}

