use std::fmt::{Display, Write};

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
    let (spec, required_types) = {
        let mut quicktype_buf = QuicktypeBuf::new();
        match data {
            Data::Struct(strukt) => {
                let args = args.with_extra(ident.clone());
                strukt.fmt_quicktype(&mut quicktype_buf, args)?
            }
            Data::Enum(enom) => enom.fmt_quicktype(&mut quicktype_buf, args)?,
            Data::Union(_) => return Err(unsupported(item, "unions")),
        }
        let QuicktypeBuf {
            spec,
            required_types,
        } = quicktype_buf;
        (spec, required_types)
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
                {
                    fn _assert_implements_quicktype() {
                        #[allow(unused)]
                        fn assert_implements_quicktype<T: ::quicktype::Quicktype>() {}
                        #(assert_implements_quicktype::<#required_types>());*
                    }
                }
                ::quicktype::TypeSpec::from(#spec)
            }
        }
    })
}

trait Quicktype {
    type Args;

    fn fmt_quicktype(&self, f: &mut QuicktypeBuf, args: Self::Args) -> Result<()>;
}

#[derive(Debug, Default)]
struct QuicktypeBuf {
    spec: String,
    required_types: Vec<Ident>,
}

impl QuicktypeBuf {
    fn new() -> Self {
        Self::default()
    }

    fn write(&mut self, to_write: impl Display) {
        write!(&mut self.spec, "{to_write}").expect("internal error: could not write to spec");
    }

    fn require_type(&mut self, ty: Ident) {
        if self.required_types.contains(&ty) {
            return;
        }
        self.required_types.push(ty);
    }
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

    fn fmt_quicktype(&self, f: &mut QuicktypeBuf, args: Self::Args) -> Result<()> {
        (*self).fmt_quicktype(f, args)
    }
}

impl Quicktype for DataStruct {
    type Args = QuicktypeArgs<Ident>;

    fn fmt_quicktype(&self, f: &mut QuicktypeBuf, args: Self::Args) -> Result<()> {
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

    fn fmt_quicktype(&self, f: &mut QuicktypeBuf, args: QuicktypeArgs) -> Result<()> {
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

    fn fmt_quicktype(&self, f: &mut QuicktypeBuf, args: QuicktypeArgs) -> Result<()> {
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

    fn fmt_quicktype(&self, f: &mut QuicktypeBuf, args: Self::Args) -> Result<()> {
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

    fn fmt_quicktype(&self, f: &mut QuicktypeBuf, args: QuicktypeArgs) -> Result<()> {
        self.as_str().fmt_quicktype(f, args)
    }
}

impl Quicktype for &str {
    type Args = QuicktypeArgs;

    fn fmt_quicktype(&self, f: &mut QuicktypeBuf, _args: QuicktypeArgs) -> Result<()> {
        f.write(format_args!("{self:?}"));
        Ok(())
    }
}

impl Quicktype for FieldsNamed {
    type Args = QuicktypeArgs;

    fn fmt_quicktype(&self, f: &mut QuicktypeBuf, args: QuicktypeArgs) -> Result<()> {
        let Self {
            brace_token: _,
            named,
        } = self;
        f.write('{');
        named.fmt_quicktype(f, args)?;
        f.write('}');
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

    fn fmt_quicktype(&self, f: &mut QuicktypeBuf, args: QuicktypeArgs<A>) -> Result<()> {
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

    fn fmt_quicktype(&self, f: &mut QuicktypeBuf, _args: QuicktypeArgs) -> Result<()> {
        f.write(", ");
        Ok(())
    }
}

impl Quicktype for Token![+] {
    type Args = QuicktypeArgs;

    fn fmt_quicktype(&self, f: &mut QuicktypeBuf, _args: QuicktypeArgs) -> Result<()> {
        f.write(" + ");
        Ok(())
    }
}

impl Quicktype for Token![|] {
    type Args = QuicktypeArgs;

    fn fmt_quicktype(&self, f: &mut QuicktypeBuf, _args: QuicktypeArgs) -> Result<()> {
        f.write(" | ");
        Ok(())
    }
}

impl Quicktype for FieldsUnnamed {
    type Args = QuicktypeArgs;

    fn fmt_quicktype(&self, f: &mut QuicktypeBuf, args: QuicktypeArgs) -> Result<()> {
        let Self {
            paren_token: _,
            unnamed,
        } = self;
        f.write('(');
        unnamed.fmt_quicktype(f, args)?;
        f.write(')');
        Ok(())
    }
}

impl Quicktype for Field {
    type Args = QuicktypeArgs;

    fn fmt_quicktype(&self, f: &mut QuicktypeBuf, args: QuicktypeArgs) -> Result<()> {
        let Self {
            attrs: _,
            vis: _,
            mutability: _,
            ident,
            colon_token: _,
            ty,
        } = self;
        if let Some(ident) = ident {
            ident.fmt_quicktype(f, args.clone().with_extra(IdentPosition::FieldName))?;
            f.write(": ");
        }
        ty.fmt_quicktype(f, args)?;
        Ok(())
    }
}

impl Quicktype for Ident {
    type Args = QuicktypeArgs<IdentPosition>;

    fn fmt_quicktype(
        &self,
        f: &mut QuicktypeBuf,
        args: QuicktypeArgs<IdentPosition>,
    ) -> Result<()> {
        let (args, ident_position) = args.split_extra();
        match ident_position {
            IdentPosition::FieldName => {
                for chr in self.to_string().chars() {
                    if chr.is_ascii_uppercase() {
                        f.write('_');
                    }
                    f.write(chr.to_ascii_lowercase());
                }
            }
            IdentPosition::TypeName => {
                if let Some(namespace) = args.namespace {
                    f.write(namespace.value());
                    f.write('.');
                }
                f.write(self.to_string());
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

    fn fmt_quicktype(&self, f: &mut QuicktypeBuf, args: QuicktypeArgs) -> Result<()> {
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

    fn fmt_quicktype(&self, f: &mut QuicktypeBuf, args: QuicktypeArgs) -> Result<()> {
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

    fn fmt_quicktype(&self, f: &mut QuicktypeBuf, args: QuicktypeArgs) -> Result<()> {
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
        f.write('(');
        inputs.fmt_quicktype(f, args.clone())?;
        f.write(") -> ");
        output.fmt_quicktype(f, args)?;
        Ok(())
    }
}

impl Quicktype for BareFnArg {
    type Args = QuicktypeArgs;

    fn fmt_quicktype(&self, f: &mut QuicktypeBuf, args: QuicktypeArgs) -> Result<()> {
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

    fn fmt_quicktype(&self, f: &mut QuicktypeBuf, args: QuicktypeArgs) -> Result<()> {
        match self {
            Self::Default => Err(unsupported(self, "inferred return types")),
            Self::Type(_, ty) => ty.fmt_quicktype(f, args),
        }
    }
}

impl Quicktype for TypeImplTrait {
    type Args = QuicktypeArgs;

    fn fmt_quicktype(&self, f: &mut QuicktypeBuf, args: QuicktypeArgs) -> Result<()> {
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

    fn fmt_quicktype(&self, f: &mut QuicktypeBuf, args: QuicktypeArgs) -> Result<()> {
        match self {
            Self::Trait(trayt) => trayt.fmt_quicktype(f, args),
            _ => Err(unsupported(self, "non-trait type bound")),
        }
    }
}

impl Quicktype for TraitBound {
    type Args = QuicktypeArgs;

    fn fmt_quicktype(&self, f: &mut QuicktypeBuf, args: QuicktypeArgs) -> Result<()> {
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

    fn fmt_quicktype(&self, f: &mut QuicktypeBuf, _args: QuicktypeArgs) -> Result<()> {
        let Self { bang_token: _ } = self;
        f.write('!');
        Ok(())
    }
}

impl Quicktype for TypeParen {
    type Args = QuicktypeArgs;

    fn fmt_quicktype(&self, f: &mut QuicktypeBuf, args: QuicktypeArgs) -> Result<()> {
        let Self {
            paren_token: _,
            elem,
        } = self;
        elem.fmt_quicktype(f, args)
    }
}

impl Quicktype for TypePath {
    type Args = QuicktypeArgs;

    fn fmt_quicktype(&self, f: &mut QuicktypeBuf, args: QuicktypeArgs) -> Result<()> {
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

    fn fmt_quicktype(&self, f: &mut QuicktypeBuf, args: QuicktypeArgs) -> Result<()> {
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
            PathArguments::None => QuicktypeType::new(ident)?.fmt_quicktype(f, args),
            PathArguments::AngleBracketed(bracketed_args) => {
                let AngleBracketedGenericArguments {
                    colon2_token: _,
                    lt_token: _,
                    args: generic_args,
                    gt_token: _,
                } = bracketed_args;
                QuicktypeType::with_generics(ident, generic_args)?.fmt_quicktype(f, args)
            }
            PathArguments::Parenthesized(p) => p.fmt_quicktype(f, args),
        }
    }
}

#[allow(clippy::large_enum_variant)]
enum QuicktypeType {
    Boolean,
    String,
    Number,
    Option(GenericArgument),
    Set(GenericArgument),
    List(GenericArgument),
    Map(GenericArgument, GenericArgument),
    Custom(Ident),
}

impl QuicktypeType {
    fn new(name: &Ident) -> Result<Self> {
        Self::with_generics_impl(name, None)
    }

    fn with_generics(
        name: &Ident,
        generics: &Punctuated<GenericArgument, Token![,]>,
    ) -> Result<Self> {
        Self::with_generics_impl(name, Some(generics))
    }

    fn with_generics_impl(
        name: &Ident,
        generics: Option<&Punctuated<GenericArgument, Token![,]>>,
    ) -> Result<Self> {
        let ident_string = name.to_string();
        let generic_args: Vec<_> = generics.into_iter().flatten().collect();
        match (generic_args.as_slice(), ident_string.as_str()) {
            ([], "bool") => Ok(Self::Boolean),
            ([], "String" | "str") => Ok(Self::String),
            (
                [],
                "u8" | "u16" | "u32" | "u64" | "u128" | "i8" | "i16" | "i32" | "i64" | "i128"
                | "usize" | "isize" | "f32" | "f64",
            ) => Ok(Self::Number),
            ([], _) => Ok(Self::Custom(name.to_owned())),
            ([elem_type], "Option") => Ok(Self::Option((*elem_type).to_owned())),
            ([elem_type], "Vec") => Ok(Self::List((*elem_type).to_owned())),
            ([elem_type], name) if name.ends_with("Set") => Ok(Self::Set((*elem_type).to_owned())),
            ([key_type, value_type], name) if name.ends_with("Map") => {
                Ok(Self::Map((*key_type).clone(), (*value_type).clone()))
            }
            _ => Err(unsupported(generics, "arbitrary generic arguments")),
        }
    }
}

impl Quicktype for QuicktypeType {
    type Args = QuicktypeArgs;

    fn fmt_quicktype(&self, f: &mut QuicktypeBuf, args: QuicktypeArgs) -> Result<()> {
        match self {
            Self::Boolean => {
                f.write("boolean");
                Ok(())
            }
            Self::String => {
                f.write("string");
                Ok(())
            }
            Self::Number => {
                f.write("number");
                Ok(())
            }
            Self::Option(elem_type) => {
                f.write('?');
                elem_type.fmt_quicktype(f, args)
            }
            Self::Set(elem_type) => {
                f.write('{');
                elem_type.fmt_quicktype(f, args)?;
                f.write('}');
                Ok(())
            }
            Self::List(elem_type) => {
                f.write('[');
                elem_type.fmt_quicktype(f, args)?;
                f.write(']');
                Ok(())
            }
            Self::Map(key_type, value_type) => {
                f.write('{');
                key_type.fmt_quicktype(f, args.clone())?;
                f.write(" -> ");
                value_type.fmt_quicktype(f, args)?;
                f.write('}');
                Ok(())
            }
            Self::Custom(ty) => {
                f.require_type(ty.clone());
                ty.fmt_quicktype(f, args.with_extra(IdentPosition::TypeName))
            }
        }
    }
}

impl Quicktype for ParenthesizedGenericArguments {
    type Args = QuicktypeArgs;

    fn fmt_quicktype(&self, f: &mut QuicktypeBuf, args: QuicktypeArgs) -> Result<()> {
        let Self {
            paren_token: _,
            inputs,
            output,
        } = self;
        f.write('(');
        inputs.fmt_quicktype(f, args.clone())?;
        f.write(") -> ");
        output.fmt_quicktype(f, args)?;
        Ok(())
    }
}

impl Quicktype for GenericArgument {
    type Args = QuicktypeArgs;

    fn fmt_quicktype(&self, f: &mut QuicktypeBuf, args: QuicktypeArgs) -> Result<()> {
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

    fn fmt_quicktype(&self, f: &mut QuicktypeBuf, args: QuicktypeArgs) -> Result<()> {
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

    fn fmt_quicktype(&self, f: &mut QuicktypeBuf, args: QuicktypeArgs) -> Result<()> {
        let Self {
            bracket_token: _,
            elem,
        } = self;
        f.write('[');
        elem.fmt_quicktype(f, args)?;
        f.write(']');
        Ok(())
    }
}

impl Quicktype for TypeTraitObject {
    type Args = QuicktypeArgs;

    fn fmt_quicktype(&self, f: &mut QuicktypeBuf, args: QuicktypeArgs) -> Result<()> {
        let Self {
            dyn_token: _,
            bounds,
        } = self;
        bounds.fmt_quicktype(f, args)
    }
}

impl Quicktype for TypeTuple {
    type Args = QuicktypeArgs;

    fn fmt_quicktype(&self, f: &mut QuicktypeBuf, args: QuicktypeArgs) -> Result<()> {
        let Self {
            paren_token: _,
            elems,
        } = self;
        f.write('(');
        elems.fmt_quicktype(f, args)?;
        f.write(')');
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
