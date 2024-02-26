use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ImplItem, Item, ItemImpl, ItemMod};

#[proc_macro_attribute]
pub fn endpoints(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut item_mod: ItemMod = parse_macro_input!(item as ItemMod);

    // Extract the name of the trait that is being Implemented.
    // The trait name is the service in the protobuffer file.
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
                                |t| {
                                    Some(
                                        t.1.segments
                                            .last()
                                            .expect("No trait implementation in module")
                                            .ident
                                            .to_string(),
                                    )
                                },
                            )
                            .unwrap_or_else(|| "".to_string()),
                    )
                } else {
                    None
                }
            })
        })
        .expect("No trait implementation in module.")
        .collect();

    // Implementations that does not implement a trait is filtered out.
    let names: Vec<String> = impl_names
        .clone()
        .into_iter()
        .filter(|item| !item.is_empty())
        .collect();

    let mut endpoints: Vec<String> = Vec::new();

    let items_impl: Vec<Item> = item_mod.clone().content.unwrap().1;

    // Will convert Item to ItemImpl
    let items_impl: Vec<ItemImpl> = items_impl
        .into_iter()
        .filter_map(|item| match item {
            Item::Impl(item_impl) => Some(item_impl),
            _ => None,
        })
        .collect();

    // Filter out impl without trait.
    let items_impl: Vec<ItemImpl> = items_impl
        .clone()
        .into_iter()
        .filter(|item| item.trait_.is_some())
        .collect();

    // Iterate over all functions in an implementation and adds the name of trait and function in Pascal case.
    for i in 0..items_impl.len() {
        let existing_functions: Vec<String> = items_impl[i]
            .items
            .iter()
            .filter_map(|item| match item {
                syn::ImplItem::Fn(function) => {
                    Some(function.sig.ident.to_string().to_case(Case::Pascal))
                }
                _ => None,
            })
            .collect();

        for function in existing_functions {
            endpoints.push(format!("{}/{}", names[i], function));
        }
    }

    // The endpoints function is constructed.
    let new_function: TokenStream = quote! {
        async fn endpoints(&self, request: tonic::Request<()>) -> std::result::Result<tonic::Response<EndpointsResponse>, tonic::Status> {
            let names = vec![#(#endpoints.to_string()),*];
            Ok(Response::new(EndpointsResponse {
                endpoints: names
            }))
        }
    }
        .into();

    // It's cursed, but what could be expected from traversing an AST without some kind of pattern.
    // A method of getting the reference to the "EcdarApiAuth" implementation
    let specific_impl: Option<&mut ItemImpl> = item_mod
        .content
        .as_mut()
        .map(|module_content| {
            module_content.1.iter_mut().find_map(|item| {
                if let Item::Impl(item_impl) = item {
                    if let Some(trait_ref) = &item_impl.trait_ {
                        if let Some(ident) = trait_ref.1.get_ident() {
                            if ident == "EcdarApiAuth" {
                                return Some(item_impl);
                            }
                        }
                    }
                }
                None
            })
        })
        .expect("No specific implementation for EcdarApiAuth");

    // Appends the function to the implementation.
    if let Some(impl_item) = specific_impl {
        impl_item
            .items
            .push(parse_macro_input!(new_function as ImplItem));
    }

    // Construct the tokens for the whole module.
    let output = quote! {#item_mod};

    output.into()
}
