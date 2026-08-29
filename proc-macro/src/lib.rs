use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_quote, ItemStruct};

#[proc_macro_attribute]
pub fn eventful(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut item = parse_macro_input!(input as ItemStruct);

    let (struct_name, internal_field_name) = if args.is_empty() {
        (item.ident.clone(), None)
    } else {
        let path: syn::Path = parse_macro_input!(args as syn::Path);
        (
            path.segments.first().unwrap().ident.clone(),
            Some(path.segments.last().unwrap().ident.clone()),
        )
    };
    let mut methods = Vec::new();

    for field in &mut item.fields {
        let Some(field_name) = &field.ident else {
            continue;
        };

        let is_event = field.attrs.iter().any(|attr| attr.path().is_ident("event"));

        if !is_event {
            continue;
        }

        // Don't leave #[event] behind, otherwise rustc will complain
        // about an unknown attribute.
        field.attrs.retain(|attr| !attr.path().is_ident("event"));

        let event_ty = field.ty.clone();

        // Rewrite:
        //
        //     on_login: OnLogin
        //
        // into:
        //
        //     on_login: Event<OnLogin>
        //
        field.ty = parse_quote! {
            Event<#event_ty>
        };

        if let Some(internal_field_name) = &internal_field_name {
            methods.push(quote! {
                pub fn #field_name<F>(&self, callback: F) -> EventHandle<#event_ty>
                where
                    F: Fn(#event_ty) + Send + Sync + 'static,
                {
                    self.#internal_field_name.#field_name.listen(callback)
                }
            });
        } else {
            methods.push(quote! {
                pub fn #field_name<F>(&self, callback: F) -> EventHandle<#event_ty>
                where
                    F: Fn(#event_ty) + Send + Sync + 'static,
                {
                    self.#field_name.listen(callback)
                }
            });
        }
    }

    quote! {
        #item

        impl #struct_name {
            #(#methods)*
        }
    }
    .into()
}
