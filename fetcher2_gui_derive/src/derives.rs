use proc_macro2::TokenStream;
use proc_macro_error::abort_call_site;
use quote::quote;
use syn::{Data, DeriveInput, Fields, PathArguments, Type};

pub fn derive_node(input: &DeriveInput) -> TokenStream {
    let name = &input.ident;
    return quote! {
        impl TreeNode for #name {
            fn children_count(&self) -> usize {
                self.children.len()
            }

            fn get_child(&self, index: usize) -> &Self {
                &self.children[index]
            }

            fn for_child_mut<V>(
                &mut self,
                index: usize,
                cb: impl FnOnce(&mut Self, usize) -> V,
            ) -> V {
                let mut new_child = self.children[index].to_owned();
                let v = cb(&mut new_child, index);
                if !new_child.same(&self.children[index]) {
                    self.children[index] = new_child;
                }
                v
            }

            fn rm_child(&mut self, index: usize) {
                self.children.remove(index);
            }
        }
    };
}

pub fn derive_root_node(input: &DeriveInput) -> TokenStream {
    let name = &input.ident;
    let child_name = find_child_node_ty(input);

    return quote! {
        impl TreeNodeRoot<#child_name> for #name {
            fn children_count(&self) -> usize {
                self.children.len()
            }

            fn get_child(&self, index: usize) -> &#child_name {
                &self.children[index]
            }

            fn for_child_mut<V>(
                &mut self,
                index: usize,
                cb: impl FnOnce(&mut #child_name, usize) -> V,
            ) -> V {
                let mut new_child = self.children[index].to_owned();
                let v = cb(&mut new_child, index);
                if !new_child.same(&self.children[index]) {
                    self.children[index] = new_child;
                }
                v
            }

            fn rm_child(&mut self, index: usize) {
                self.children.remove(index);
            }
        }
    };
}

fn find_child_node_ty(input: &DeriveInput) -> TokenStream {
    if let Data::Struct(data_struct) = &input.data {
        if let Fields::Named(fields) = &data_struct.fields {
            if let Some(child_field) = fields.named.iter().find(|field| {
                if let Some(name) = &field.ident {
                    name == "children"
                } else {
                    false
                }
            }) {
                if let Type::Path(path) = &child_field.ty {
                    if let PathArguments::AngleBracketed(x) = &path.path.segments[0].arguments {
                        let name = &x.args;
                        quote! {#name}
                    } else {
                        abort_call_site!("Not struct")
                    }
                } else {
                    abort_call_site!("Not struct")
                }
            } else {
                abort_call_site!("Not struct")
            }
        } else {
            abort_call_site!("Not struct")
        }
    } else {
        abort_call_site!("Not struct")
    }
}
