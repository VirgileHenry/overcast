use quote::{format_ident, quote};


#[proc_macro_derive(Serializable)]
pub fn derive_serializable(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_derive_serializable(ast)
}

fn impl_derive_serializable(ast: syn::DeriveInput) -> proc_macro::TokenStream {
    let ident = ast.ident;
    
    match ast.data {
        syn::Data::Struct(struct_data) => {

            let size = generate_serialized_size(&struct_data.fields);
            let fields_serialization = generate_fields_serialization(&struct_data.fields, false);
            let fields_deserialization = generate_fields_deserialization(&struct_data.fields);
            let result_struct = generate_struct_res(&ident, &struct_data.fields);

            quote!(
                impl Serializable for #ident {
                    const MAX_BIN_SIZE: usize = { #(#size)* };
                    fn serialize(&self, into: &mut [u8]) -> Result<usize, ()> {
                        if into.len() < Self::MAX_BIN_SIZE { Err(()) }
                        else {
                            let mut offset = 0;
                            #( #fields_serialization )*
                            Ok(offset)
                        }
                    }
                    fn deserialize(from: &[u8]) -> Result<Self, ()> {
                        if from.len() < Self::MAX_BIN_SIZE { Err(()) }
                        else {
                            let mut offset = 0;
                            #( #fields_deserialization )*
                            Ok( #result_struct )
                        }
                    }
                }
            )
        },
        syn::Data::Enum(enum_data) => {
            let variants_indices: Vec<proc_macro2::TokenStream> = enum_data.variants.iter().enumerate().map(|(i, _)| {
                let index_as_u8 = i as u8;
                quote!( #index_as_u8 )
            }).collect::<Vec<_>>();
            let sizes = enum_data.variants.iter().map(|variant| {
                generate_serialized_size(&variant.fields)
            }).collect::<Vec<_>>();
            let fields_serializations = enum_data.variants.iter().map(|variant| {
                generate_fields_serialization(&variant.fields, true)
            });
            let fields_deserializations = enum_data.variants.iter().map(|variant| {
                generate_fields_deserialization(&variant.fields)
            });
            let results_variants = enum_data.variants.iter().map(|variant| {
                generate_struct_res(&variant.ident, &variant.fields)
            }).collect::<Vec<_>>();
            let create_variants = fields_deserializations
                .into_iter()
                .zip(results_variants.iter())
                .map(|(field_deser, result)| {
                    quote!(
                        #( #field_deser )*
                        Ok( #ident::#result )
                    )
                }).collect::<Vec<_>>();

            let mut max_size = quote!( 0 );
            for size in sizes {
                max_size = quote!( Self::macr_gen_const_max( #max_size, #(#size)* ) )
            }

            let enum_serialization = quote!( 
                match self {
                    #(
                        #ident::#results_variants => {
                            <u8 as Serializable>::serialize(&#variants_indices, &mut into[0..1])?;
                            let mut offset = 1;
                            #(
                                #fields_serializations
                            )*
                            Ok(offset)
                        }
                    )*
                    _ => unreachable!(),
                }
            );
            
            let enum_deserialization = quote!(
                match variant_index {
                    #(
                        #variants_indices => {
                            #create_variants
                        }
                    )*
                    _ => Err(())
                }
            );

            quote!(
                impl #ident {
                    const fn macr_gen_const_max(a: usize, b: usize) -> usize {
                        if a > b { a } else { b }
                    }
                }

                impl Serializable for #ident {
                    const MAX_BIN_SIZE: usize = { 1 + #max_size };
                    fn serialize(&self, into: &mut [u8]) -> Result<usize, ()> {
                        if into.len() < Self::MAX_BIN_SIZE { Err(()) }
                        else {
                            #enum_serialization
                        }
                    }
                    fn deserialize(from: &[u8]) -> Result<Self, ()> {
                        if from.len() < Self::MAX_BIN_SIZE { Err(()) }
                        else {
                            let variant_index: u8 = <u8 as Serializable>::deserialize(&from[0..1])?;
                            let mut offset = 1;
                            #enum_deserialization
                        }
                    }
                }
            )
        },
        _ => quote!(
            compile_error!("Serializable derive macro not yet implemented for unions")
        )
    }.into()
}

fn generate_serialized_size(fields: &syn::Fields) -> Vec<proc_macro2::TokenStream> {
    match fields {
        syn::Fields::Named(fields) => std::iter::once(quote!( 0 )).chain(fields.named.iter().map(|field| {
            let ty = &field.ty;
            quote!(  + <#ty as Serializable>::MAX_BIN_SIZE )
        })).collect::<Vec<_>>(),
        syn::Fields::Unnamed(fields) => std::iter::once(quote!( 0 )).chain(fields.unnamed.iter().map(|field| {
            let ty = &field.ty;
            quote!(  + <#ty as Serializable>::MAX_BIN_SIZE )
        })).collect::<Vec<_>>(),
        syn::Fields::Unit => vec![quote!( 0 )],
    }
}

fn generate_fields_serialization(fields: &syn::Fields, use_fields_generated_name: bool) -> Vec<proc_macro2::TokenStream> {
    match fields {
        syn::Fields::Named(fields) => fields.named.iter().filter(|field| {
            field.ident.is_some()
        }).map(|field| {
            let field_ident = if use_fields_generated_name {
                let ident = field.ident.as_ref().unwrap(); 
                quote!( #ident )
            }
            else {
                let ident = field.ident.as_ref().unwrap(); 
                quote! ( &self.#ident )
            };
            let field_type = &field.ty;
            quote!(
                <#field_type as Serializable>::serialize(#field_ident, &mut into[offset..offset+<#field_type as Serializable>::MAX_BIN_SIZE])?;
                offset += <#field_type as Serializable>::MAX_BIN_SIZE;
            )
        }).collect::<Vec<_>>(),
        syn::Fields::Unnamed(fields) => fields.unnamed.iter().enumerate().map(|(i, field)| {
            let index = if use_fields_generated_name {
                let ident = format_ident!("field_{i}");
                quote!( #ident )
            }
            else {
                let ident = syn::Index::from(i);
                quote! ( &self.#ident )
            };
            let field_type = &field.ty;
            quote!(
                <#field_type as Serializable>::serialize(#index, &mut into[offset..offset+<#field_type as Serializable>::MAX_BIN_SIZE])?;
                offset += <#field_type as Serializable>::MAX_BIN_SIZE;
            )
        }).collect::<Vec<_>>(),
        syn::Fields::Unit => Vec::with_capacity(0),
    }
}

fn generate_fields_deserialization(fields: &syn::Fields) -> Vec<proc_macro2::TokenStream> {
    match fields {
        syn::Fields::Named(fields) => fields.named.iter().filter(|field| {
            field.ident.is_some()
        }).map(|field| {
            let decl_ident = field.ident.as_ref().unwrap();
            let field_type = &field.ty;
            quote!(
                let #decl_ident = <#field_type as Serializable>::deserialize(&from[offset..offset+<#field_type as Serializable>::MAX_BIN_SIZE])?;
                offset += <#field_type as Serializable>::MAX_BIN_SIZE;
            )
        }).collect::<Vec<_>>(),
        syn::Fields::Unnamed(fields) => fields.unnamed.iter().enumerate().map(|(i, field)| {
            let decl_ident = format_ident!("field_{}", i);
            let field_type = &field.ty;
            quote!(
                let #decl_ident = <#field_type as Serializable>::deserialize(&from[offset..offset+<#field_type as Serializable>::MAX_BIN_SIZE])?;
                offset += <#field_type as Serializable>::MAX_BIN_SIZE;
            )
        }).collect::<Vec<_>>(),
        syn::Fields::Unit => Vec::with_capacity(0),
    }
}

fn generate_struct_res(ident: &syn::Ident, fields: &syn::Fields) -> proc_macro2::TokenStream {
    match fields {
        syn::Fields::Named(fields) => {
            let fields_in_struct = fields.named.iter().map(|field| {
                let field_ident = field.ident.as_ref().unwrap();
                quote!( #field_ident )
            });
            quote!( #ident { #(#fields_in_struct,)* } )
        }
        syn::Fields::Unnamed(fields) => {
            let fields_in_struct = fields.unnamed.iter().enumerate().map(|(i, _)| {
                let field_ident = format_ident!("field_{}", i);
                quote!( #field_ident )
            });
            quote!( #ident ( #(#fields_in_struct,)* ) )
        }
        syn::Fields::Unit => quote!( #ident )
    }
}
