use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, Ident, Type};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let _ = input;

    let ast = syn::parse(input).expect("Can't parse the input stream");

    impl_derive_builder(&ast)
}

fn get_generic_from_opt(ty: &Type) -> Option<&Type> {
    match ty {
        syn::Type::Path(p) => {
            if p.qself.is_some() {
                return None;
            }

            if p.path.leading_colon.is_some() {
                return None;
            }

            let segment = p.path.segments.first().unwrap();

            if segment.ident.to_string() != "Option" {
                return None;
            }

            match &segment.arguments {
                syn::PathArguments::AngleBracketed(arg) => {
                    if arg.args.len() != 1 {
                        return None;
                    } else {
                        return match arg.args.first().unwrap() {
                            syn::GenericArgument::Type(gty) => Some(gty),
                            _ => None,
                        };
                    }
                }
                _ => {
                    return None;
                }
            }
        }
        _ => return None,
    }
}

fn impl_derive_builder(derive_input: &DeriveInput) -> TokenStream {
    let struct_name = &derive_input.ident;
    let builder_name = Ident::new(&format!("{}Builder", struct_name), struct_name.span());

    let fields = match &derive_input.data {
        syn::Data::Struct(data) => &data.fields,
        _ => unimplemented!(),
    };

    let fields_builder = fields.iter().map(|field| {
        let name = field.ident.as_ref().unwrap();
        let ty = &field.ty;

        if get_generic_from_opt(ty).is_some() {
            quote!(
                #name: #ty
            )
        } else {
            quote!(
                #name: std::option::Option<#ty>,
            )
        }
    });

    let init_fields = fields.iter().map(|f| {
        let name = f.ident.as_ref().unwrap();

        quote!(
            #name: std::option::Option::<_>::None,
        )
    });

    let method = fields.iter().map(|f| {
        let name = f.ident.as_ref().unwrap();
        let patameter_type = &f.ty;

        if let Some(gty) = get_generic_from_opt(patameter_type) {
            quote!(
                pub fn #name(&mut self, #name: #gty) -> &mut Self {
                    self.#name = Some(#name);
                    self
                }
            )
        } else {
            quote!(
                pub fn #name(&mut self, #name: #patameter_type) -> &mut Self {
                    self.#name = Some(#name);
                    self
                }
            )
        }
    });

    let init_struct_from_builder = fields.iter().map(|f| {
        let name = f.ident.as_ref().unwrap();
        let ty = &f.ty;
        if let Some(_) = get_generic_from_opt(ty) {
            quote!(
             #name: self.#name.clone()
            )
        } else {
            quote!(
                #name: self.#name.as_ref().ok_or("missing field".to_string())?.clone(),
            )
        }
    });

    let gen = quote! {

        use std::error::Error;

        pub struct #builder_name {
            #(#fields_builder)*
        }

        impl #builder_name {
            #(#method)*
        }

        impl #builder_name {
            pub fn build(&mut self) -> Result<#struct_name, Box<dyn Error>> {

                Ok(#struct_name {
                    #(#init_struct_from_builder)*
                })
            }
        }

        impl #struct_name {
            pub fn builder() -> #builder_name {
                #builder_name {
                    #(#init_fields)*
                }
            }
        }

    };

    gen.into()
}
