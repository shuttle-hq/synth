/// This crate supplies two procedural macros, one attribute and one bang.
///
/// The whole language bindings have three moving parts:
///
/// * The `FromValue`, `FromValueRef` and `IntoValue` traits from the `lang_bindings`
/// crate are there to get Rust values from and into koto (or borrow into them).
/// Since we bind our custom types as `ExternalValue`s, we need to implement the
/// eponymous trait. We also have a `StaticExternalValue` that doesn't need to
/// allocate
/// * impl blocks mostly get collected into virtual dispatch tables (VTables). Those
/// are static `ValueMap`s from function name to a wrapped function that takes care of
/// the value conversions
/// * associated or free functions go into a module, which is a table that is also
/// available from koto; functions without either a type or given module name go into
/// the `synth` module for now. You can manually override the module with 
/// `#[bindlang("module_name")]`. Inherent constructors of both structs and enums are
/// also bound if both the type and all relevant fields are `pub`
///
/// The [`#[bindlang]`]() proc_macro attribute is there to collect all items, and the
/// `bindlang_main!(..)` macro expands to the collected items. It creates both a number
/// of static VTables and the `bindlang_init(&mut ValueMap)` function that sets up the
/// modules. 

extern crate proc_macro;
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;
use quote::quote;
use syn::*;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

fn ident(s: &str) -> Ident {
    syn::parse_str(s).unwrap()
}

#[derive(Clone, Copy, Debug)]
enum BindingMode {
    ByValue,
    ByRef,
    ByRefMut,
}

impl From<&Type> for BindingMode {
    fn from(ty: &Type) -> Self {
        match ty {
            Type::Reference(TypeReference { mutability: Some(_), .. }) => Self::ByRefMut,
            Type::Reference(TypeReference { mutability: None, .. }) => Self::ByRef,
            _ => Self::ByValue,
        }
    }
}

impl From<&Field> for BindingMode {
    fn from(f: &Field) -> Self {
        Self::from(&f.ty)
    }
}

impl From<&FnArg> for BindingMode {
    fn from(f: &FnArg) -> Self {
        match &f {
            FnArg::Receiver(Receiver { reference: Some(_), mutability: Some(_), ..}) => Self::ByRefMut,
            FnArg::Receiver(Receiver { reference: Some(_), mutability: None, ..}) => Self::ByRef,
            FnArg::Typed(PatType { ty, .. }) => Self::from(&**ty),
            _ => Self::ByValue,
        }
    }
}

#[derive(Debug)]
enum MethodArgs {
    /// unit constructors (e.g. `MyStruct`, `My)Enum::UnitVariant`
    Unit,
    /// function call or tuple struct / variant, `self_ty` contains the self type for method calls
    Tuple {
        args: Vec<BindingMode>,
        self_ty: Option<String>,
    },
    /// named arguments, in given order
    Named { names: Vec<(String, BindingMode)> },
}

/// all we need to wrap a method or constructor
#[derive(Debug)]
struct MethodSig {
    /// This is the name on the Rust side of things (which may be different from koto's name, e.g.
    /// `MyEnum::MyVariant` or `<MyType as MyTrait>::my_function`)
    path: String,
    /// The arguments. There are 3 variants: Unit for unit structs/variants, Tuple for functions
    /// and tuple structs/variants and Named for named structs/variants
    args: MethodArgs,
    /// a stringified representation of the required arguments for error reporting
    inputs: String,
}

fn is_public(vis: &Visibility) -> bool {
    matches!(vis, Visibility::Public(_))
}

fn is_field_public(field: &Field) -> bool {
    is_public(&field.vis)
}

impl MethodSig {
    fn from_fields(path: String, fields: &Fields, is_enum: bool) -> Option<Self> {
        let result = Some(MethodSig {
            inputs: path.clone(), //TODO: improve this
            path,
            args: match fields {
                Fields::Named(FieldsNamed { named, .. }) => {
                    if is_enum || named.iter().all(is_field_public) {
                        MethodArgs::Named {
                            names: named
                                .iter()
                                .map(|f| (f.ident.as_ref().unwrap().to_string(), BindingMode::from(f)))
                                .collect(),
                        }
                    } else {
                        return None;
                    }
                }
                Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                    if is_enum || unnamed.iter().all(is_field_public) {
                        MethodArgs::Tuple {
                            args: unnamed.iter().map(BindingMode::from).collect(),
                            self_ty: None,
                        }
                    } else {
                        return None;
                    }
                }
                Fields::Unit => MethodArgs::Unit,
            },
        });
        result
    }

    fn from_sig(sig: &Signature, item_impl: Option<&ItemImpl>) -> Self {
        let args = MethodArgs::Tuple {
            args: sig.inputs.iter().map(BindingMode::from).collect(),
            self_ty: sig.receiver().and_then(|_| {
                item_impl.map(|i| {
                    let ty = &*i.self_ty;
                    (quote! { #ty }).to_string()
                })
            }),
        };
        let fn_ident = &sig.ident;
        let path = match &item_impl {
           // Some(_) if sig.receiver().is_some() => quote! { #fn_ident },
            Some(&ItemImpl {
                ref self_ty,
                trait_: Some((_, ref trait_ty, _)),
                ..
            }) => quote! { <#self_ty as #trait_ty>::#fn_ident },
            Some(&ItemImpl {
                ref self_ty,
                trait_: None,
                ..
            }) => quote! { #self_ty::#fn_ident },
            None => quote! { #fn_ident },
        }
        .to_string();
        MethodSig {
            path,
            args,
            inputs: (quote! { #sig }).to_string(),
        }
    }

    // create a tuple expr of (call, is_method)
    //
    // variants:
    // * named struct / enum variant
    // * unit struct / enum variant
    // * tuple struct / enum variant
    // * plain function call / plain trait function call
    // * method call

    fn to_tuple(&self, koto_name: &str) -> Expr {
        let path: Expr = parse_str(&self.path).unwrap();
        match &self.args {
            MethodArgs::Tuple { args, self_ty } => {
                let is_method = self_ty.is_some();
                let inputs = &self.inputs;
                let idents: Vec<_> = (0..args.len()).map(|i| (ident(&format!("a{}", i)), ident(&format!("v{}", i)))).collect();
                let inner_idents = idents.iter().map(|(_, v)| v);
                let mut expr: Expr = parse_quote! { #path(#(#inner_idents),*) };
                for (i, ((a, v), mode)) in idents.iter().zip(args.iter()).enumerate().rev() {
                    expr = match mode {
                        BindingMode::ByRef => {
                            parse_quote! { 
                                ::lang_bindings::RefFromValue::ref_from_value(
                                    &::lang_bindings::KeyPath::Index(#i, None),
                                    #a,
                                    |#v| #expr
                                )?
                            }
                        },
                        BindingMode::ByRefMut => {
                            parse_quote! { 
                                ::lang_bindings::RefFromValue::ref_mut_from_value(
                                    &::lang_bindings::KeyPath::Index(#i, None),
                                    #a,
                                    |#v| #expr
                                )?
                            }
                        },
                        BindingMode::ByValue => {
                            parse_quote! {
                                match ::lang_bindings::FromValue::from_value(
                                    &::lang_bindings::KeyPath::Index(#i, None),
                                    #a
                                )? { 
                                    #v => #expr
                                }
                            }
                        }
                    };
                }
                let outer_idents = idents.iter().map(|(a, _)| a);
                parse_quote! {
                    (
                        #koto_name,
                        ::koto_runtime::ExternalFunction::new(|vm, args| match vm.get_args(args) {
                            [#(#outer_idents),*] => ::lang_bindings::IntoValue::into_value(#expr),
                            args => ::lang_bindings::fn_type_error(#koto_name, #inputs, args),
                        }, #is_method)
                    )
                }
            }
            MethodArgs::Named { names } => {
                let idents: Vec<_> = names.iter().map(|n| ident(&n.0)).collect();
                let mut name_iter = names.iter().map(|(s, _)| s);
                let mut inputs = name_iter.next().map_or(String::new(), ToOwned::to_owned);
                for name in name_iter {
                    inputs += ", ";
                    inputs += name;
                }
                parse_quote! {
                    (
                        #koto_name,
                        ::koto_runtime::ExternalFunction::new(|vm, args| {
                            match vm.get_args(args) {
                                [#(#idents),*] => ::lang_bindings::IntoValue::into_value(#path {
                                    #(#idents: ::lang_bindings::FromValue::from_value(
                                        &::lang_bindings::KeyPath::Field(
                                            std::borrow::Cow::Borrowed(stringify!(#idents)),
                                            None
                                        ),
                                        #idents
                                    )?),*
                                }),
                                args => ::lang_bindings::fn_type_error(#koto_name, #inputs, args),
                            }
                        }, false)
                    )
                }
            }
            MethodArgs::Unit => parse_quote! {
                (
                    #koto_name,
                    ::koto_runtime::ExternalFunction::new(|vm, args| {
                        match vm.get_args(args) {
                            [] => ::lang_bindings::IntoValue::into_value(#path),
                            args => ::lang_bindings::fn_type_error(#koto_name, "()", args),
                        }
                    }, false)
                )
            },
        }
    }
}

// We need some structures to keep stuff around
#[derive(Default)]
struct Context {
    bare_fns: HashMap<String, MethodSig>,
    modules: HashMap<String, HashMap<String, MethodSig>>,
    vtables: HashMap<String, HashMap<String, MethodSig>>,
    types: HashMap<String, String>,
}

lazy_static::lazy_static! {
    static ref CONTEXT: Arc<Mutex<Context>> = Arc::new(Mutex::new(Context::default()));
}

fn bind_sig(sig: &Signature, module: Option<String>, item_impl: Option<&ItemImpl>) {
    let ctx = &mut CONTEXT.lock().unwrap();
    let fn_ident = &sig.ident;
    if let Some(i) = item_impl {
        let self_ty = &*i.self_ty;
        let self_ty_str = quote! { #self_ty }.to_string();
        if sig.receiver().is_some() {
            &mut ctx.vtables
        } else {
            &mut ctx.modules
        }
        .entry(self_ty_str)
        .or_insert_with(HashMap::new)
        .insert(fn_ident.to_string(), MethodSig::from_sig(sig, item_impl));
    } else {
        let mut module_entry;
        if let Some(m) = module {
            module_entry = ctx.modules.entry(m).or_insert_with(HashMap::new);
            &mut module_entry
        } else {
            &mut ctx.bare_fns
        }
        .insert(fn_ident.to_string(), MethodSig::from_sig(sig, None));
    }
}

fn get_attr_parens(attr: &Attribute) -> String {
    attr.tokens
        .to_string()
        .trim_matches(&['(', ')'][..])
        .to_owned()
}

fn get_module(attrs: &[Attribute]) -> Option<String> {
    attrs.iter().find_map(|a| {
        if a.path.get_ident().map_or(false, |i| i == "bindlang") {
            Some(get_attr_parens(a))
        } else {
            None
        }
    })
}

// derive, (name, args)
static DERIVES: &[(&str, (&str, &[BindingMode]))] = &[
    ("Default", ("default", &[])),
    ("Clone", ("clone", &[BindingMode::ByRef])),
    ("Display", ("to_string", &[BindingMode::ByRef])),
    //TODO: map other traits
];

fn get_derives(ty: String, attrs: &[Attribute]) {
    for attr in attrs {
        if attr.path.get_ident().map_or(false, |i| i == "derive") {
            for derive in get_attr_parens(attr).split(',') {
                let derive = derive.trim_matches(char::is_whitespace);
                for (trt, (name, args)) in DERIVES {
                    if trt == &derive {
                        let name = *name;
                        let args = args.to_vec();
                        let path = format!("<{} as {}>::{}", ty, trt, name);
                        let ctx = &mut CONTEXT.lock().unwrap();
                        ctx.modules
                            .entry(ty.clone())
                            .or_insert_with(HashMap::new)
                            .insert(
                                name.to_owned(),
                                MethodSig {
                                    path,
                                    args: MethodArgs::Tuple {
                                        args,
                                        self_ty: None,
                                    },
                                    inputs: String::new(),
                                },
                            );
                        break;
                    };
                }
            }
        }
    }
}

fn get_pub_constructors(i: &Item) {
    match i {
        Item::Struct(ItemStruct {
            vis,
            ident,
            //generics,
            fields,
            ..
        }) => {
            if !is_public(vis) {
                return;
            }
            let ty = ident.to_string();
            if let Some(method_sig) = MethodSig::from_fields(ty.clone(), fields, false) {
                CONTEXT.lock().unwrap().bare_fns.insert(ty, method_sig);
            }
        }
        Item::Enum(ItemEnum {
            vis,
            ident,
            variants,
            ..
        }) => {
            if !is_public(vis) {
                return;
            }
            let ty = ident.to_string();
            for variant in variants.iter() {
                let path: String = ty.clone() + "::" + &variant.ident.to_string();
                if let Some(sig) = MethodSig::from_fields(path, &variant.fields, true) {
                    CONTEXT
                        .lock()
                        .unwrap()
                        .modules
                        .entry(ty.clone())
                        .or_insert_with(HashMap::new)
                        .insert(variant.ident.to_string(), sig);
                }
            }
        }
        _ => {}
    }
}

fn create_map(items: &HashMap<String, MethodSig>) -> Expr {
    let members = items.iter().map(|(name, sig)| sig.to_tuple(name));
    parse_quote! {
        ::koto_runtime::ValueMap::with_data(IntoIterator::into_iter([
            #(#members),*
        ]).map(|(k,v): (&str, ::koto_runtime::ExternalFunction)| (
            ::koto_runtime::ValueKey::from(::koto_runtime::Value::Str(k.into())),
            ::koto_runtime::Value::ExternalFunction(v)
        )).collect::<::koto_runtime::ValueHashMap>())
    }
}

fn create_vtable(vtable: &Ident, items: &HashMap<String, MethodSig>) -> Item {
    let map = create_map(items);
    parse_quote! {
        lazy_static::lazy_static! {
            static ref #vtable: ::koto_runtime::ValueMap = #map;
        }
    }
}

fn create_module(prelude: &Ident, name: &str, items: &HashMap<String, MethodSig>) -> Stmt {
    let map = create_map(items);
    let lname = name.to_lowercase();
    parse_quote! {
        #prelude.add_map(#lname, #map);
    }
}

//TODO: We may want to allow generics at some point, but we'd need to introduce a new type to parse them
fn create_binding(ty_name: &str, _generics: &str, ty_vtable: &Ident) -> impl Iterator<Item = Item> {
    let ty: Ident = ident(ty_name);
    IntoIterator::into_iter(
        //if generics.is_empty() {
        [
            parse_quote! {
                impl ::lang_bindings::StaticExternalValue for #ty {
                    fn type_str() -> &'static str { #ty_name }
                }
            },
            parse_quote! {
                impl ::koto_runtime::ExternalValue for #ty {
                    fn value_type(&self) -> String {
                        String::from(#ty_name)
                    }
                }
            },
            parse_quote! {
                impl ::lang_bindings::FromValue for #ty {
                    fn from_value(
                        key_path: &::lang_bindings::KeyPath<'_>,
                        value: &::koto_runtime::Value,
                    ) -> std::result::Result<Self, ::koto_runtime::RuntimeError> {
                        if let Some(result) = <Self as ::lang_bindings::CustomFromValue>::opt_from_value(value) {
                            return Ok(result);
                        }
                        if let ::koto_runtime::Value::ExternalValue(exval, ..) = value {
                            if let Some(v) = exval.as_ref().write().downcast_mut::<Self>() {
                                Ok(v.clone())
                            } else {
                                ::lang_bindings::wrong_type(
                                    <Self as ::lang_bindings::StaticExternalValue>::type_str(),
                                    key_path,
                                    &value,
                                )
                            }
                        } else {
                            ::lang_bindings::not_external_value(key_path, &*value)
                        }
                    }
                }
            },
            parse_quote! {
                impl ::lang_bindings::RefFromValue for #ty {
                    fn ref_from_value<R, F: Fn(&Self) -> R>(
                        key_path: &::lang_bindings::KeyPath<'_>,
                        value: &::koto_runtime::Value,
                        f: F,
                    ) -> std::result::Result<R, ::koto_runtime::RuntimeError> {
                        if let ::koto_runtime::Value::ExternalValue(exval, ..) = value {
                            if let Some(v) = exval.as_ref().read().downcast_ref::<Self>() {
                                Ok(f(v))
                            } else {
                                ::lang_bindings::wrong_type(
                                    <Self as ::lang_bindings::StaticExternalValue>::type_str(),
                                    key_path,
                                    &value,
                                )
                            }
                        } else {
                            ::lang_bindings::not_external_value(key_path, &*value)
                        }
                    }

                    fn ref_mut_from_value<R, F: for<'r> Fn(&'r Self) -> R>(
                        key_path: &::lang_bindings::KeyPath<'_>,
                        value: &::koto_runtime::Value,
                        f: F,
                    ) -> std::result::Result<R, ::koto_runtime::RuntimeError> {
                        if let ::koto_runtime::Value::ExternalValue(exval, ..) = value {
                            if let Some(v) = exval.as_ref().write().downcast_mut::<Self>() {
                                Ok(f(v))
                            } else {
                                ::lang_bindings::wrong_type(
                                    <Self as ::lang_bindings::StaticExternalValue>::type_str(),
                                    key_path,
                                    &value,
                                )
                            }
                        } else {
                            ::lang_bindings::not_external_value(key_path, &*value)
                        }
                    }
                }
            },
            parse_quote! {
                impl ::lang_bindings::IntoValue for #ty {
                    fn into_value(self)
                    -> std::result::Result<::koto_runtime::Value, ::koto_runtime::RuntimeError> {
                        Ok(::koto_runtime::Value::make_external_value(self, #ty_vtable.clone()))
                    }
                }
            },
        ],
    )
}

#[proc_macro]
pub fn bindlang_main(mut code: TokenStream) -> TokenStream {
    let Context {
        ref bare_fns,
        ref modules,
        ref vtables,
        ref types,
    } = *CONTEXT.lock().unwrap();
    let prelude = ident("prelude");
    let vtable_idents = vtables
        .keys()
        .map(|ty| (ty.to_string(), vtable_ident(ty)))
        .collect::<HashMap<String, Ident>>();
    let vtable_items = vtables
        .iter()
        .map(|(name, items)| create_vtable(&vtable_idents[name], items));
    let type_bindings = types
        .iter()
        .flat_map(|(name, generics)| create_binding(name, generics, &vtable_idents[name]));
    let prelude_map = create_map(bare_fns);
    let module_stmts = modules
        .iter()
        .map(|(name, items)| create_module(&prelude, name, items));
    //TODO we may insert the "synth" string below in the _code tokens
    code.extend(TokenStream::from(quote! {
        #(#vtable_items)*
        #(#type_bindings)*
        fn bindlang_init(#prelude: &mut ::koto_runtime::ValueMap) {
            #prelude.add_map("synth", #prelude_map);
            #(#module_stmts)*
        }
    }));
    code
}

fn vtable_ident(ty: &str) -> Ident {
    ident(&format!("__BINDLANG_VTABLE_{}__", ty))
}

#[proc_macro_attribute]
pub fn bindlang(_attrs: TokenStream, code: TokenStream) -> TokenStream {
    let code_cloned = code.clone();
    let input = parse_macro_input!(code_cloned as Item);
    get_pub_constructors(&input);
    match &input {
        //TODO: bind trait impls
        Item::Impl(item_impl) => {
            for item in &item_impl.items {
                if let ImplItem::Method(ImplItemMethod { ref sig, .. }) = item {
                    bind_sig(sig, get_module(&item_impl.attrs), Some(item_impl));
                }
            }
        }
        Item::Fn(ItemFn {
            ref attrs, ref sig, ..
        }) => {
            bind_sig(sig, get_module(attrs), None);
        }
        Item::Struct(ItemStruct {
            attrs,
            ident: ty,
            generics: Generics { params, .. },
            ..
        })
        | Item::Enum(ItemEnum {
            attrs,
            ident: ty,
            generics: Generics { params, .. },
            ..
        }) => {
            // record the type, derives and generics (as String)
            let ty_string = ty.to_string();
            get_derives(ty_string.clone(), attrs);
            let mut ctx = CONTEXT.lock().unwrap();
            if !ctx.vtables.contains_key(&ty_string) {
                ctx.vtables.insert(ty_string.clone(), HashMap::new());
            }
            ctx.types.insert(ty_string, quote!(#params).to_string());
        }
        //TODO: Do we want or need to bind other items? E.g. statics?
        _ => (), //TODO: Report a usable error
    }
    // we emit the code as is
    code
}
