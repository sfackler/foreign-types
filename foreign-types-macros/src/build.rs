use proc_macro2::TokenStream;
use quote::quote;
use syn::{Path, Ident};

use crate::parse::{Input, ForeignType};

fn ref_name(input: &ForeignType) -> Ident {
    Ident::new(&format!("{}Ref", input.name), input.name.span())
}

pub fn build(input: Input) -> TokenStream {
    let types = input.types.iter().map(|t| build_foreign_type(&input.crate_, t));
    quote! {
        #(#types)*
    }
}

fn build_foreign_type(crate_: &Path, input: &ForeignType) -> TokenStream {
    let decls = build_decls(crate_, input);
    let oibits = build_oibits(crate_, input);
    let foreign_impls = build_foreign_impls(crate_, input);
    let drop_impl = build_drop_impl(crate_, input);
    let deref_impls = build_deref_impls(crate_, input);
    let borrow_impls = build_borrow_impls(crate_, input);
    let as_ref_impls = build_as_ref_impls(crate_, input);
    let clone_impl = build_clone_impl(crate_, input);
    let to_owned_impl = build_to_owned_impl(crate_, input);

    quote! {
        #decls
        #oibits
        #foreign_impls
        #drop_impl
        #deref_impls
        #borrow_impls
        #as_ref_impls
        #clone_impl
        #to_owned_impl
    }
}

fn build_decls(crate_: &Path, input: &ForeignType) -> TokenStream {
    let attrs = &input.attrs;
    let vis = &input.visibility;
    let name = &input.name;
    let ctype = &input.ctype;
    let ref_name = ref_name(input);
    let ref_docs = format!("A borrowed reference to a [`{name}`](struct.{}.html).", name = name);

    quote! {
        #(#attrs)*
        #vis struct #name(#crate_::export::NonNull<#ctype>);

        #[doc = #ref_docs]
        #vis struct #ref_name(#crate_::Opaque);
    }
}

fn build_oibits(crate_: &Path, input: &ForeignType) -> TokenStream {
    let oibits = input.oibits.iter().map(|t| build_oibit(crate_, input, t));

    quote! {
        #(#oibits)*
    }
}

fn build_oibit(crate_: &Path, input: &ForeignType, oibit: &Ident) -> TokenStream {
    let name = &input.name;
    let ref_name = ref_name(input);

    quote! {
        unsafe impl #crate_::export::#oibit for #name {}
        unsafe impl #crate_::export::#oibit for #ref_name {}
    }
}

fn build_foreign_impls(crate_: &Path, input: &ForeignType) -> TokenStream {
    let name = &input.name;
    let ctype = &input.ctype;
    let ref_name = ref_name(input);

    quote! {
        impl #crate_::ForeignType for #name {
            type CType = #ctype;
            type Ref = #ref_name;

            #[inline]
            unsafe fn from_ptr(ptr: *mut #ctype) -> #name {
                debug_assert!(!ptr.is_null());
                #name(<#crate_::export::NonNull<_>>::new_unchecked(ptr))
            }

            #[inline]
            fn as_ptr(&self) -> *mut #ctype {
                <#crate_::export::NonNull<_>>::as_ptr(self.0)
            }
        }

        impl #crate_::ForeignTypeRef for #ref_name {
            type CType = #ctype;
        }
    }
}

fn build_drop_impl(crate_: &Path, input: &ForeignType) -> TokenStream {
    let name = &input.name;
    let drop = &input.drop;

    quote! {
        impl #crate_::export::Drop for #name {
            #[inline]
            fn drop(&mut self) {
                unsafe {
                    #drop(#crate_::ForeignType::as_ptr(self));
                }
            }
        }
    }
}

fn build_deref_impls(crate_: &Path, input: &ForeignType) -> TokenStream {
    let name = &input.name;
    let ref_name = ref_name(input);

    quote! {
        impl #crate_::export::Deref for #name {
            type Target = #ref_name;

            #[inline]
            fn deref(&self) -> &#ref_name {
                unsafe {
                    #crate_::ForeignTypeRef::from_ptr(#crate_::ForeignType::as_ptr(self))
                }
            }
        }

        impl #crate_::export::DerefMut for #name {
            #[inline]
            fn deref_mut(&mut self) -> &mut #ref_name {
                unsafe {
                    #crate_::ForeignTypeRef::from_ptr_mut(#crate_::ForeignType::as_ptr(self))
                }
            }
        }
    }
}

fn build_borrow_impls(crate_: &Path, input: &ForeignType) -> TokenStream {
    let name = &input.name;
    let ref_name = ref_name(input);

    quote! {
        impl #crate_::export::Borrow<#ref_name> for #name {
            #[inline]
            fn borrow(&self) -> &#ref_name {
                &**self
            }
        }

        impl #crate_::export::BorrowMut<#ref_name> for #name {
            #[inline]
            fn borrow_mut(&mut self) -> &mut #ref_name {
                &mut **self
            }
        }
    }
}

fn build_as_ref_impls(crate_: &Path, input: &ForeignType) -> TokenStream {
    let name = &input.name;
    let ref_name = ref_name(input);

    quote! {
        impl #crate_::export::AsRef<#ref_name> for #name {
            #[inline]
            fn as_ref(&self) -> &#ref_name {
                &**self
            }
        }

        impl #crate_::export::AsMut<#ref_name> for #name {
            #[inline]
            fn as_mut(&mut self) -> &mut #ref_name {
                &mut **self
            }
        }
    }
}

fn build_clone_impl(crate_: &Path, input: &ForeignType) -> TokenStream {
    let clone = match &input.clone {
        Some(clone) => clone,
        None => return quote!(),
    };
    let name = &input.name;

    quote! {
        impl #crate_::export::Clone for #name {
            #[inline]
            fn clone(&self) -> #name {
                unsafe {
                    let ptr = #clone(#crate_::ForeignType::as_ptr(self));
                    #crate_::ForeignType::from_ptr(ptr)
                }
            }
        }
    }
}

#[cfg(feature = "std")]
fn build_to_owned_impl(crate_: &Path, input: &ForeignType) -> TokenStream {
    let clone = match &input.clone {
        Some(clone) => clone,
        None => return quote!(),
    };
    let name = &input.name;
    let ref_name = ref_name(input);

    quote! {
        impl #crate_::export::ToOwned for #ref_name {
            type Owned = #name;

            #[inline]
            fn to_owned(&self) -> #name {
                unsafe {
                    let ptr = #clone(#crate_::ForeignTypeRef::as_ptr(self));
                    #crate_::ForeignType::from_ptr(ptr)
                }
            }
        }
    }
}

#[cfg(not(feature = "std"))]
fn build_to_owned_impl(_: &Path, _: &ForeignType) -> TokenStream {
    quote!()
}
