use std::fmt::Write;

use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::parse::Parse;
use syn::punctuated::{Pair, Punctuated};
use syn::{
    AngleBracketedGenericArguments, AttrStyle, Attribute, BareFnArg, Data, DataEnum, DataStruct,
    DeriveInput, Error, Field, Fields, FieldsNamed, FieldsUnnamed, GenericArgument, Ident, LitStr,
    Meta, ParenthesizedGenericArguments, Path, PathArguments, PathSegment, QSelf, Result,
    ReturnType, Token, TraitBound, Type, TypeArray, TypeBareFn, TypeImplTrait, TypeNever,
    TypeParamBound, TypeParen, TypePath, TypeReference, TypeSlice, TypeTraitObject, TypeTuple,
    Variant, parse_macro_input,
};

#[proc_macro_derive(Quicktype, attributes(quicktype))]
pub fn quicktype(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match quicktype_impl(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.into_compile_error().into(),
    }
}

fn quicktype_impl(item: DeriveInput) -> Result<TokenStream> {
    let DeriveInput {
        attrs,
        vis: _,
        ident,
        generics,
        data,
    } = &item;

    let AttrArgs { namespace } = AttrArgs::try_from(attrs.as_slice())?;

    if generics.lt_token.is_some() && !generics.params.is_empty() {
        return Err(unsupported(generics, "generic parameters"));
    }

    let args = match &namespace {
        Some(namespace) => QuicktypeArgs::with_namespace(namespace.clone()),
        None => QuicktypeArgs::new(),
    };
    let namespace = match namespace {
        Some(namespace) => quote! { Some(::quicktype::Namespace::from(#namespace)) },
        None => quote!(None),
    };
    let unqualified_name = ident.to_string();
    let quicktype_spec = {
        let mut quicktype_type = String::new();
        match data {
            Data::Struct(strukt) => {
                let args = args.with_extra(ident.clone());
                strukt.fmt_quicktype(&mut quicktype_type, args)?
            }
            Data::Enum(enom) => enom.fmt_quicktype(&mut quicktype_type, args)?,
            Data::Union(_) => return Err(unsupported(item, "unions")),
        }
        quicktype_type
    };
    Ok(quote! {
        impl ::quicktype::Quicktype for #ident {
            fn type_name() -> ::quicktype::TypeName {
                ::quicktype::TypeName {
                    namespace: #namespace,
                    unqualified_name: ::quicktype::UnqualifiedTypeName::from(#unqualified_name),
                }
            }

            fn type_spec() -> ::quicktype::TypeSpec {
                ::quicktype::TypeSpec::from(#quicktype_spec)
            }
        }
    })
}

trait Quicktype {
    type Args;

    fn fmt_quicktype(&self, f: &mut String, args: Self::Args) -> Result<()>;
}

#[derive(Clone)]
struct QuicktypeArgs<A = ()> {
    namespace: Option<LitStr>,
    extra: A,
}

impl QuicktypeArgs {
    fn new() -> Self {
        Self {
            namespace: None,
            extra: (),
        }
    }

    fn with_namespace(namespace: LitStr) -> Self {
        Self {
            namespace: Some(namespace),
            extra: (),
        }
    }

    fn with_extra<A>(self, extra: A) -> QuicktypeArgs<A> {
        let Self {
            namespace,
            extra: _,
        } = self;
        QuicktypeArgs { namespace, extra }
    }
}

impl<A> QuicktypeArgs<A> {
    fn split_extra(self) -> (QuicktypeArgs<()>, A) {
        let Self { namespace, extra } = self;
        let args = QuicktypeArgs {
            namespace,
            extra: (),
        };
        (args, extra)
    }
}

impl<T: Quicktype> Quicktype for &T {
    type Args = T::Args;

    fn fmt_quicktype(&self, f: &mut String, args: Self::Args) -> Result<()> {
        (*self).fmt_quicktype(f, args)
    }
}

impl Quicktype for DataStruct {
    type Args = QuicktypeArgs<Ident>;

    fn fmt_quicktype(&self, f: &mut String, args: Self::Args) -> Result<()> {
        let Self {
            struct_token: _,
            fields,
            semi_token: _,
        } = self;
        fields.fmt_quicktype(f, args)
    }
}

impl Quicktype for DataEnum {
    type Args = QuicktypeArgs;

    fn fmt_quicktype(&self, f: &mut String, args: QuicktypeArgs) -> Result<()> {
        let Self {
            enum_token: _,
            brace_token: _,
            variants,
        } = self;
        let variants: Punctuated<_, Token![|]> = variants.iter().collect();
        variants.fmt_quicktype(f, args)
    }
}

impl Quicktype for Variant {
    type Args = QuicktypeArgs;

    fn fmt_quicktype(&self, f: &mut String, args: QuicktypeArgs) -> Result<()> {
        let Self {
            attrs: _,
            ident,
            fields,
            discriminant: _,
        } = self;
        let args = args.with_extra(ident.clone());
        fields.fmt_quicktype(f, args)
    }
}

impl Quicktype for Fields {
    type Args = QuicktypeArgs<Ident>;

    fn fmt_quicktype(&self, f: &mut String, args: Self::Args) -> Result<()> {
        let (args, name) = args.split_extra();
        match self {
            Self::Named(named) => named.fmt_quicktype(f, args),
            Self::Unnamed(unnamed) => unnamed.fmt_quicktype(f, args),
            Self::Unit => name.to_string().fmt_quicktype(f, args),
        }
    }
}

impl Quicktype for String {
    type Args = QuicktypeArgs;

    fn fmt_quicktype(&self, f: &mut String, args: QuicktypeArgs) -> Result<()> {
        self.as_str().fmt_quicktype(f, args)
    }
}

impl Quicktype for &str {
    type Args = QuicktypeArgs;

    fn fmt_quicktype(&self, f: &mut String, _args: QuicktypeArgs) -> Result<()> {
        write!(f, "{self:?}").ok();
        Ok(())
    }
}

impl Quicktype for FieldsNamed {
    type Args = QuicktypeArgs;

    fn fmt_quicktype(&self, f: &mut String, args: QuicktypeArgs) -> Result<()> {
        let Self {
            brace_token: _,
            named,
        } = self;
        f.push('{');
        named.fmt_quicktype(f, args)?;
        f.push('}');
        Ok(())
    }
}

impl<A, T, P> Quicktype for Punctuated<T, P>
where
    A: Clone,
    T: Quicktype<Args = QuicktypeArgs<A>>,
    P: Quicktype<Args = QuicktypeArgs<()>>,
{
    type Args = QuicktypeArgs<A>;

    fn fmt_quicktype(&self, f: &mut String, args: QuicktypeArgs<A>) -> Result<()> {
        let mut prev_punct: Option<&P> = None;

        for pair in self.pairs() {
            match pair {
                Pair::Punctuated(t, p) => {
                    if let Some(prev_punct) = prev_punct {
                        let (args, _) = args.clone().split_extra();
                        prev_punct.fmt_quicktype(f, args)?;
                    }
                    prev_punct = Some(p);
                    t.fmt_quicktype(f, args.clone())?;
                }
                Pair::End(t) => t.fmt_quicktype(f, args.clone())?,
            }
        }
        Ok(())
    }
}

impl Quicktype for Token![,] {
    type Args = QuicktypeArgs;

    fn fmt_quicktype(&self, f: &mut String, _args: QuicktypeArgs) -> Result<()> {
        f.push_str(", ");
        Ok(())
    }
}

impl Quicktype for Token![+] {
    type Args = QuicktypeArgs;

    fn fmt_quicktype(&self, f: &mut String, _args: QuicktypeArgs) -> Result<()> {
        f.push_str(" + ");
        Ok(())
    }
}

impl Quicktype for Token![|] {
    type Args = QuicktypeArgs;

    fn fmt_quicktype(&self, f: &mut String, _args: QuicktypeArgs) -> Result<()> {
        f.push_str(" | ");
        Ok(())
    }
}

impl Quicktype for FieldsUnnamed {
    type Args = QuicktypeArgs;

    fn fmt_quicktype(&self, f: &mut String, args: QuicktypeArgs) -> Result<()> {
        let Self {
            paren_token: _,
            unnamed,
        } = self;
        f.push('(');
        unnamed.fmt_quicktype(f, args)?;
        f.push(')');
        Ok(())
    }
}

impl Quicktype for Field {
    type Args = QuicktypeArgs;

    fn fmt_quicktype(&self, f: &mut String, args: QuicktypeArgs) -> Result<()> {
        let Self {
            attrs: _, // TODO(kcza): check serde attrs
            vis: _,
            mutability: _,
            ident,
            colon_token: _,
            ty,
        } = self;
        if let Some(ident) = ident {
            ident.fmt_quicktype(f, args.clone().with_extra(IdentPosition::FieldName))?;
            f.push_str(": ");
        }
        ty.fmt_quicktype(f, args)?;
        Ok(())
    }
}

impl Quicktype for Ident {
    type Args = QuicktypeArgs<IdentPosition>;

    fn fmt_quicktype(&self, f: &mut String, args: QuicktypeArgs<IdentPosition>) -> Result<()> {
        let (args, ident_position) = args.split_extra();
        match ident_position {
            IdentPosition::FieldName => f.push_str(&self.to_string()),
            IdentPosition::TypeName => {
                if let Some(namespace) = args.namespace {
                    f.push_str(&namespace.value());
                    f.push('.');
                }
                f.push_str(&self.to_string());
            }
        }
        Ok(())
    }
}

enum IdentPosition {
    FieldName,
    TypeName,
}

impl Quicktype for Type {
    type Args = QuicktypeArgs;

    fn fmt_quicktype(&self, f: &mut String, args: QuicktypeArgs) -> Result<()> {
        match self {
            Type::Array(array) => array.fmt_quicktype(f, args),
            Type::BareFn(bare_fn) => bare_fn.fmt_quicktype(f, args),
            Type::Group(group) => Err(unsupported(group, "group types")),
            Type::ImplTrait(impl_trait) => impl_trait.fmt_quicktype(f, args),
            Type::Infer(infer) => Err(unsupported(infer, "type inference")),
            Type::Macro(makro) => Err(unsupported(makro, "macro types")),
            Type::Never(never) => never.fmt_quicktype(f, args),
            Type::Paren(type_paren) => type_paren.fmt_quicktype(f, args),
            Type::Path(path) => path.fmt_quicktype(f, args),
            Type::Ptr(ptr) => Err(unsupported(ptr, "raw pointer types")),
            Type::Reference(reff) => reff.fmt_quicktype(f, args),
            Type::Slice(slice) => slice.fmt_quicktype(f, args),
            Type::TraitObject(trait_object) => trait_object.fmt_quicktype(f, args),
            Type::Tuple(tuple) => tuple.fmt_quicktype(f, args),
            Type::Verbatim(verbatim) => Err(unsupported(verbatim, "non-syn-supported types")),
            _ => Err(unsupported(self, "unknown type")),
        }
    }
}

impl Quicktype for TypeArray {
    type Args = QuicktypeArgs;

    fn fmt_quicktype(&self, f: &mut String, args: QuicktypeArgs) -> Result<()> {
        let Self {
            bracket_token: _,
            elem,
            semi_token: _,
            len: _,
        } = self;
        elem.fmt_quicktype(f, args)
    }
}

impl Quicktype for TypeBareFn {
    type Args = QuicktypeArgs;

    fn fmt_quicktype(&self, f: &mut String, args: QuicktypeArgs) -> Result<()> {
        let Self {
            lifetimes: _,
            unsafety: _,
            abi: _,
            fn_token: _,
            paren_token: _,
            inputs,
            variadic,
            output,
        } = self;
        if let Some(variadic) = variadic {
            return Err(unsupported(variadic, "variadic function parameters"));
        }
        f.push('(');
        inputs.fmt_quicktype(f, args.clone())?;
        f.push_str(") -> ");
        output.fmt_quicktype(f, args)?;
        Ok(())
    }
}

impl Quicktype for BareFnArg {
    type Args = QuicktypeArgs;

    fn fmt_quicktype(&self, f: &mut String, args: QuicktypeArgs) -> Result<()> {
        let Self {
            attrs: _,
            name: _,
            ty,
        } = self;
        ty.fmt_quicktype(f, args)
    }
}

impl Quicktype for ReturnType {
    type Args = QuicktypeArgs;

    fn fmt_quicktype(&self, f: &mut String, args: QuicktypeArgs) -> Result<()> {
        match self {
            Self::Default => Err(unsupported(self, "inferred return types")),
            Self::Type(_, ty) => ty.fmt_quicktype(f, args),
        }
    }
}

impl Quicktype for TypeImplTrait {
    type Args = QuicktypeArgs;

    fn fmt_quicktype(&self, f: &mut String, args: QuicktypeArgs) -> Result<()> {
        let Self {
            impl_token: _,
            bounds,
        } = self;
        let valid_bounds: Punctuated<_, Token![+]> = bounds
            .iter()
            .filter_map(|bound| match bound {
                TypeParamBound::Trait(trayt) => Some(trayt),
                _ => None,
            })
            .collect();
        valid_bounds.fmt_quicktype(f, args)
    }
}

impl Quicktype for TypeParamBound {
    type Args = QuicktypeArgs;

    fn fmt_quicktype(&self, f: &mut String, args: QuicktypeArgs) -> Result<()> {
        match self {
            Self::Trait(trayt) => trayt.fmt_quicktype(f, args),
            _ => Err(unsupported(self, "non-trait type bound")),
        }
    }
}

impl Quicktype for TraitBound {
    type Args = QuicktypeArgs;

    fn fmt_quicktype(&self, f: &mut String, args: QuicktypeArgs) -> Result<()> {
        let Self {
            paren_token: _,
            modifier: _,
            lifetimes: _,
            path,
        } = self;
        path.fmt_quicktype(f, args)
    }
}

impl Quicktype for TypeNever {
    type Args = QuicktypeArgs;

    fn fmt_quicktype(&self, f: &mut String, _args: QuicktypeArgs) -> Result<()> {
        let Self { bang_token: _ } = self;
        f.push('!');
        Ok(())
    }
}

impl Quicktype for TypeParen {
    type Args = QuicktypeArgs;

    fn fmt_quicktype(&self, f: &mut String, args: QuicktypeArgs) -> Result<()> {
        let Self {
            paren_token: _,
            elem,
        } = self;
        elem.fmt_quicktype(f, args)
    }
}

impl Quicktype for TypePath {
    type Args = QuicktypeArgs;

    fn fmt_quicktype(&self, f: &mut String, args: QuicktypeArgs) -> Result<()> {
        let Self { qself, path } = self;
        if let Some(qself) = qself {
            let QSelf {
                lt_token: _,
                ty,
                position: _,
                as_token: _,
                gt_token: _,
            } = qself;
            return Err(unsupported(ty, "self"));
        }
        path.fmt_quicktype(f, args)
    }
}

impl Quicktype for Path {
    type Args = QuicktypeArgs;

    fn fmt_quicktype(&self, f: &mut String, args: QuicktypeArgs) -> Result<()> {
        if let Some(ident) = self.get_ident() {
            return ident.fmt_quicktype(f, args.with_extra(IdentPosition::TypeName));
        }

        let Self {
            leading_colon: _,
            segments,
        } = self;

        if segments.len() > 1 {
            return Err(Error::new_spanned(self, "too many path segments"));
        }
        let segment = match segments.last() {
            Some(segment) => segment,
            None => return Err(Error::new_spanned(self, "too few path segments")),
        };
        let PathSegment { ident, arguments } = segment;
        match arguments {
            PathArguments::None => ident.fmt_quicktype(f, args.with_extra(IdentPosition::TypeName)),
            PathArguments::AngleBracketed(bracketed_args) => {
                let AngleBracketedGenericArguments {
                    colon2_token: _,
                    lt_token: _,
                    args: generic_args,
                    gt_token: _,
                } = bracketed_args;
                let ident_name = ident.to_string();

                // Special cases.
                if generic_args.len() == 1 && ident_name == "Option" {
                    f.push('?');
                    generic_args[0].fmt_quicktype(f, args)?;
                    return Ok(());
                }
                if generic_args.len() == 1 && ident_name == "Vec" {
                    f.push('[');
                    generic_args[0].fmt_quicktype(f, args)?;
                    f.push(']');
                    return Ok(());
                }
                if generic_args.len() == 1 && ident_name.ends_with("Set") {
                    f.push('{');
                    generic_args[0].fmt_quicktype(f, args)?;
                    f.push('}');
                    return Ok(());
                }
                if generic_args.len() == 2 && ident_name.ends_with("Map") {
                    f.push('{');
                    generic_args[0].fmt_quicktype(f, args.clone())?;
                    f.push_str(" -> ");
                    generic_args[1].fmt_quicktype(f, args)?;
                    f.push('}');
                    return Ok(());
                }
                if generic_args.is_empty() {
                    generic_args[0].fmt_quicktype(f, args)?;
                    return Ok(());
                }
                Err(unsupported(generic_args, "arbitrary generic arguments"))
            }
            PathArguments::Parenthesized(p) => p.fmt_quicktype(f, args),
        }
    }
}

impl Quicktype for ParenthesizedGenericArguments {
    type Args = QuicktypeArgs;

    fn fmt_quicktype(&self, f: &mut String, args: QuicktypeArgs) -> Result<()> {
        let Self {
            paren_token: _,
            inputs,
            output,
        } = self;
        f.push('(');
        inputs.fmt_quicktype(f, args.clone())?;
        f.push_str(") -> ");
        output.fmt_quicktype(f, args)?;
        Ok(())
    }
}

impl Quicktype for GenericArgument {
    type Args = QuicktypeArgs;

    fn fmt_quicktype(&self, f: &mut String, args: QuicktypeArgs) -> Result<()> {
        match self {
            Self::Lifetime(lifetime) => Err(unsupported(lifetime, "lifetime arguments")),
            Self::Type(ty) => ty.fmt_quicktype(f, args),
            Self::Const(expr) => Err(unsupported(expr, "const expr type arguments")),
            Self::AssocType(assoc_type) => {
                Err(unsupported(assoc_type, "associated type arguments"))
            }
            Self::AssocConst(assoc_const) => Err(unsupported(
                assoc_const,
                "associated constant type arguments",
            )),
            Self::Constraint(constraint) => {
                Err(unsupported(constraint, "type constraint arguments"))
            }
            _ => Err(unsupported(self, "unrecognised generic argument")),
        }
    }
}

impl Quicktype for TypeReference {
    type Args = QuicktypeArgs;

    fn fmt_quicktype(&self, f: &mut String, args: QuicktypeArgs) -> Result<()> {
        let Self {
            and_token: _,
            lifetime: _,
            mutability: _,
            elem,
        } = self;
        elem.fmt_quicktype(f, args)
    }
}

impl Quicktype for TypeSlice {
    type Args = QuicktypeArgs;

    fn fmt_quicktype(&self, f: &mut String, args: QuicktypeArgs) -> Result<()> {
        let Self {
            bracket_token: _,
            elem,
        } = self;
        f.push('[');
        elem.fmt_quicktype(f, args)?;
        f.push(']');
        Ok(())
    }
}

impl Quicktype for TypeTraitObject {
    type Args = QuicktypeArgs;

    fn fmt_quicktype(&self, f: &mut String, args: QuicktypeArgs) -> Result<()> {
        let Self {
            dyn_token: _,
            bounds,
        } = self;
        bounds.fmt_quicktype(f, args)
    }
}

impl Quicktype for TypeTuple {
    type Args = QuicktypeArgs;

    fn fmt_quicktype(&self, f: &mut String, args: QuicktypeArgs) -> Result<()> {
        let Self {
            paren_token: _,
            elems,
        } = self;
        f.push('(');
        elems.fmt_quicktype(f, args)?;
        f.push(')');
        Ok(())
    }
}

fn unsupported(loc: impl ToTokens, name: &str) -> Error {
    Error::new_spanned(loc, format!("{name} not supported"))
}

#[derive(Default)]
struct AttrArgs {
    namespace: Option<LitStr>,
}

impl TryFrom<&[Attribute]> for AttrArgs {
    type Error = Error;

    fn try_from(attrs: &[Attribute]) -> Result<Self> {
        let mut ret = Self::default();
        for attr in attrs {
            let Attribute {
                pound_token: _,
                style,
                bracket_token: _,
                meta,
            } = attr;
            if !matches!(style, AttrStyle::Outer) {
                return Err(Error::new_spanned(
                    attr,
                    "only outer attributes are supported",
                ));
            }
            let list = match meta {
                Meta::List(list) => list,
                Meta::Path(_) | Meta::NameValue(_) => continue,
            };
            match list.path.get_ident() {
                Some(ident) if ident == "quicktype" => ident,
                _ => continue,
            };
            let args =
                list.parse_args_with(Punctuated::<QuicktypeAttrArg, Token![,]>::parse_terminated)?;
            for arg in args {
                match arg {
                    QuicktypeAttrArg::Namespace {
                        _namespace: _,
                        _eq: _,
                        name,
                    } => ret.namespace = Some(name),
                }
            }
        }
        Ok(ret)
    }
}

#[derive(Debug)]
enum QuicktypeAttrArg {
    Namespace {
        _namespace: kw::namespace,
        _eq: Token![=],
        name: LitStr,
    },
}

impl Parse for QuicktypeAttrArg {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::namespace) {
            Ok(Self::Namespace {
                _namespace: input.parse()?,
                _eq: input.parse()?,
                name: input.parse()?,
            })
        } else {
            Err(lookahead.error())
        }
    }
}

mod kw {
    syn::custom_keyword!(namespace);
}
