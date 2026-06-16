//! chanlun 信号注册 proc-macro。
//!
//! 第三方代码声明：`#[signal]` 注册机制参考 czsc 项目
//! （https://github.com/waditu/czsc，Apache License 2.0），已简化适配
//! （无 category / TaCache，签名固定为 fn(&观察者, &HashMap<String, Value>) -> Vec<Signal>）。

use proc_macro::TokenStream;
use quote::quote;
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::{Expr, ExprLit, ItemFn, Lit, Meta, Token};

/// `#[signal(name = "foo_V230101", template = "{freq}_D1_foo")]`
///
/// 校验：函数名含 `_V<数字>`；`name` 与函数名一致；`name`/`template` 非空。
/// 生成：一个 `static` SignalDescriptor + `inventory::submit!`。
///
/// 路径：默认 `crate::signal::registry::`（chanlun crate 内部使用）。
/// 外部 crate 使用需指定 `crate_path = "::chanlun"`。
#[proc_macro_attribute]
pub fn signal(attr: TokenStream, item: TokenStream) -> TokenStream {
    let parser = Punctuated::<Meta, Token![,]>::parse_terminated;
    let metas = match parser.parse(attr) {
        Ok(m) => m,
        Err(e) => return e.to_compile_error().into(),
    };

    let mut name: Option<String> = None;
    let mut template: Option<String> = None;
    let mut crate_path: Option<String> = None;
    for m in metas {
        if let Meta::NameValue(nv) = m
            && let Some(ident) = nv.path.get_ident()
            && let Expr::Lit(ExprLit { lit: Lit::Str(v), .. }) = nv.value
        {
            match ident.to_string().as_str() {
                "name" => name = Some(v.value()),
                "template" => template = Some(v.value()),
                "crate_path" => crate_path = Some(v.value()),
                _ => {}
            }
        }
    }

    let f: ItemFn = match syn::parse(item) {
        Ok(v) => v,
        Err(e) => return e.to_compile_error().into(),
    };

    let name = name.unwrap_or_default();
    let template = template.unwrap_or_default();
    let fn_ident = &f.sig.ident;
    let fn_name = fn_ident.to_string();

    let mut errors = Vec::new();
    if name.is_empty() || template.is_empty() {
        errors.push(quote! { compile_error!("#[signal] name/template 不能为空"); });
    }
    if name != fn_name {
        errors.push(quote! { compile_error!("#[signal] name 必须与函数名一致"); });
    }
    // 函数名须含 _V<数字>
    let 有版本 = fn_name
        .rsplit_once("_V")
        .map(|(_, v)| !v.is_empty() && v.chars().all(|c| c.is_ascii_digit()))
        .unwrap_or(false);
    if !有版本 {
        errors.push(quote! { compile_error!("#[signal] 函数名必须含 _V<版本号>，如 foo_V230101"); });
    }

    if !errors.is_empty() {
        let errs = errors.into_iter();
        return quote! { #(#errs)* }.into();
    }

    let descriptor_ident = syn::Ident::new(
        &format!("__SIG_DESC_{}", fn_name).to_uppercase(),
        fn_ident.span(),
    );

    let path = crate_path.unwrap_or_else(|| "crate".to_string());
    let _registry_path: syn::Path = syn::parse_str(&format!("{path}::signal::registry")).unwrap();
    let signal_fn: syn::Type = syn::parse_str(&format!("{path}::signal::registry::SignalFn")).unwrap();
    let signal_desc: syn::Type = syn::parse_str(&format!("{path}::signal::registry::SignalDescriptor")).unwrap();

    let expanded = quote! {
        #f

        #[allow(non_upper_case_globals)]
        static #descriptor_ident: #signal_desc =
            #signal_desc {
                name: #name,
                template: #template,
                func: #fn_ident as #signal_fn,
            };

        inventory::submit! { #descriptor_ident }
    };
    expanded.into()
}
