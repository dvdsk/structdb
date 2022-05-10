use proc_macro2::TokenStream;
use quote::quote_spanned;
use syn::spanned::Spanned;
use syn::{Ident, Type};

pub(super) fn setter(field_ident: &Ident, field_type: &Type, key: &str) -> TokenStream {
    let setter = Ident::new(&format!("set_{}", field_ident), field_ident.span());
    let span = field_type.span();

    quote_spanned! {span=>
        #[allow(dead_code)]
        pub fn #setter(&self, position: &#field_type) -> std::result::Result<(), dbstruct::Error> {
            let bytes = bincode::serialize(position)
                .map_err(dbstruct::Error::Serializing)?;
            self.tree.insert(#key, bytes)?;
            Ok(())
        }
    }
}

pub(super) fn getter(field_ident: &Ident, field_type: &Type, key: &str) -> TokenStream {
    let getter = field_ident.clone();
    let span = field_type.span();

    quote_spanned! {span=>
        /// getter for #ident
        /// # Errors
        /// TODO
        #[allow(dead_code)]
        pub fn #getter(&self) -> std::result::Result<#field_type, dbstruct::Error> {
            let default_val = None;
            match self.tree.get(#key)? {
                Some(bytes) => Ok(bincode::deserialize(&bytes).map_err(dbstruct::Error::DeSerializing)?),
                None => Ok(default_val),
            }
        }
    }
}

pub(super) fn update(field_ident: &Ident, field_type: &Type, key: &str) -> TokenStream {
    let update = Ident::new(&format!("update_{}", field_ident), field_ident.span());
    let span = field_type.span();

    quote_spanned! {span=>
        /// # Errors
        /// returns an error incase de or re-serializing failed, in which case the
        /// value of the member in the array will not have changed.
        #[allow(dead_code)]
        pub fn #update(&self, op: impl FnMut(#field_type) -> #field_type + Clone)
            -> std::result::Result<(), dbstruct::Error> {
            let default_val = None;

            let mut res = Ok(());
            let update = |old: Option<&[u8]>| {
                let old = old.unwrap_or(None);
                match bincode::deserialize(old) {
                    Err(e) => {
                        res = Err(dbstruct::Error::DeSerializing(e));
                        Some(old.to_vec())
                    }
                    Ok(v) => {
                        let new = op.clone()(v);
                        match bincode::serialize(&new) {
                            Ok(new_bytes) => Some(new_bytes),
                            Err(e) => {
                                res = Err(dbstruct::Error::Serializing(e));
                                Some(old.to_vec())
                            }
                        }
                    }
                }
            };
            self.tree.update_and_fetch(#key, update)?;
            Ok(())
        }
    }
}

pub(super) fn compare_and_swap(field_ident: &Ident, field_type: &Type, key: &str) -> TokenStream {
    let compare_and_swap = Ident::new(
        &format!("compare_and_swap_{}", field_ident),
        field_ident.span(),
    );
    let span = field_type.span();

    quote_spanned! {span=>
        #[allow(dead_code)]
        pub fn #compare_and_swap(&self, old: #field_type, new: #field_type)
            -> std::result::Result<
                std::result::Result<(), dbstruct::CompareAndSwapError<#field_type>>,
            dbstruct::Error> {
            let old = bincode::serialize(&old).map_err(dbstruct::Error::Serializing)?;
            let new = bincode::serialize(&new).map_err(dbstruct::Error::Serializing)?;
            Ok(match self.tree.compare_and_swap(#key, Some(old), Some(new))? {
                Ok(()) => Ok(()),
                Err(e) => Err(e.try_into()?),
            })
        }
    }
}
