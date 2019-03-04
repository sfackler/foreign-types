use syn::parse::{self, Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token;
use syn::{braced, Attribute, ExprPath, Ident, Path, Token, Visibility};

mod kw {
    syn::custom_keyword!(Sync);
    syn::custom_keyword!(Send);
    syn::custom_keyword!(CType);
    syn::custom_keyword!(drop);
    syn::custom_keyword!(clone);
}

pub struct Input {
    pub crate_: Path,
    pub types: Vec<ForeignType>,
}

impl Parse for Input {
    fn parse(input: ParseStream) -> parse::Result<Input> {
        let crate_ = input.parse()?;
        let mut types = vec![];
        while !input.is_empty() {
            types.push(input.parse()?);
        }

        Ok(Input { crate_, types })
    }
}

pub struct ForeignType {
    pub attrs: Vec<Attribute>,
    pub visibility: Visibility,
    pub name: Ident,
    pub oibits: Punctuated<Ident, Token![+]>,
    pub ctype: ExprPath,
    pub drop: ExprPath,
    pub clone: Option<ExprPath>,
}

impl Parse for ForeignType {
    fn parse(input: ParseStream) -> parse::Result<ForeignType> {
        let attrs = input.call(Attribute::parse_outer)?;
        let visibility = input.parse()?;
        input.parse::<Token![type]>()?;
        let name = input.parse()?;
        let oibits = input.call(parse_oibits)?;
        let inner;
        braced!(inner in input);
        let ctype = inner.call(parse_ctype)?;
        let drop = inner.call(parse_fn::<kw::drop>)?;
        let clone = if inner.is_empty() {
            None
        } else {
            Some(inner.call(parse_fn::<kw::clone>)?)
        };

        Ok(ForeignType {
            attrs,
            visibility,
            name,
            oibits,
            ctype,
            drop,
            clone,
        })
    }
}

fn parse_oibit(input: ParseStream) -> parse::Result<Ident> {
    let lookahead = input.lookahead1();
    if lookahead.peek(kw::Sync) || lookahead.peek(kw::Send) {
        input.parse()
    } else {
        Err(lookahead.error())
    }
}

fn parse_oibits(input: ParseStream) -> parse::Result<Punctuated<Ident, Token![+]>> {
    let mut out = Punctuated::new();

    if input.parse::<Option<Token![:]>>()?.is_some() {
        loop {
            out.push_value(input.call(parse_oibit)?);
            if input.peek(token::Brace) {
                break;
            }
            out.push_punct(input.parse()?);
            if input.peek(token::Brace) {
                break;
            }
        }
    }

    Ok(out)
}

fn parse_ctype(input: ParseStream) -> parse::Result<ExprPath> {
    input.parse::<Token![type]>()?;
    input.parse::<kw::CType>()?;
    input.parse::<Token![=]>()?;
    let path = input.parse()?;
    input.parse::<Token![;]>()?;
    Ok(path)
}

fn parse_fn<T: Parse>(input: ParseStream) -> parse::Result<ExprPath> {
    input.parse::<Token![fn]>()?;
    input.parse::<T>()?;
    input.parse::<Token![=]>()?;
    let path = input.parse()?;
    input.parse::<Token![;]>()?;
    Ok(path)
}
