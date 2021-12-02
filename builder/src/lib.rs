use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, Ident};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let _ = input;

    let ast = syn::parse(input).expect("Can't parse the input stream");

    impl_derive_builder(&ast)
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

        quote!(
            #name: std::option::Option<#ty>,
        )
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

        quote!(
            pub fn #name(&mut self, #name: #patameter_type) -> &mut Self {
                self.#name = Some(#name);
                self
            }
        )
    });

    let gen = quote! {

        pub struct #builder_name {
            #(#fields_builder)*
        }

        impl #builder_name {
            #(#method)*
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
