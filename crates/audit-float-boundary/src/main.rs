#![forbid(unsafe_code)]

use proc_macro2::Span;
use std::env;
use std::fs;
use std::path::{Component, Path, PathBuf};
use syn::visit::Visit;
use syn::{
    Attribute, Expr, ExprAssign, ExprBinary, ExprCall, ExprGroup, ExprParen, ExprPath, ExprUnary,
    File, ForeignItem, ImplItem, Item, Lit, Meta, Path as SynPath, TraitItem, TypePath,
};

const GATE_TEST: u8 = 1;
const GATE_FLOAT_INGEST: u8 = 2;

#[derive(Clone, Debug)]
struct Hit {
    file: String,
    line: usize,
    ty: &'static str,
    gate: u8,
}

#[derive(Default)]
struct Summary {
    total_hits: usize,
    allow_bin: usize,
    allow_test: usize,
    allow_float_ingest: usize,
    warn_other: usize,
    core_leak: usize,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if !matches!(args.as_slice(), [_, flag, mode] if flag == "--mode" && mode == "phase5b") {
        eprintln!("usage: audit-float-boundary --mode phase5b");
        std::process::exit(2);
    }

    let root = PathBuf::from("crates");
    if !root.is_dir() {
        eprintln!("error: crates/ directory not found");
        std::process::exit(2);
    }

    let mut files = Vec::new();
    collect_rs_files(&root, &mut files);
    files.sort();

    let mut hits = Vec::new();
    for file in files {
        let src = match fs::read_to_string(&file) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("error: failed reading {}: {}", display_rel(&file), e);
                std::process::exit(2);
            }
        };
        let ast = match syn::parse_file(&src) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("error: failed parsing {}: {}", display_rel(&file), e);
                std::process::exit(2);
            }
        };
        analyze_file(&file, &ast, &mut hits);
    }

    let mut summary = Summary::default();
    for hit in hits {
        summary.total_hits += 1;
        if is_bin_path(&hit.file) {
            summary.allow_bin += 1;
            continue;
        }
        if is_test_path(&hit.file) {
            summary.allow_test += 1;
            continue;
        }
        if (hit.gate & GATE_TEST) != 0 {
            summary.allow_test += 1;
            continue;
        }
        if (hit.gate & GATE_FLOAT_INGEST) != 0 {
            summary.allow_float_ingest += 1;
            continue;
        }

        summary.warn_other += 1;
        if is_core_path(&hit.file) {
            summary.core_leak += 1;
            println!(
                "CORE-LEAK file={} line={} type={}",
                hit.file, hit.line, hit.ty
            );
        } else {
            println!("WARN file={} line={} type={}", hit.file, hit.line, hit.ty);
        }
    }

    println!("Phase 5B summary:");
    println!("total_hits={}", summary.total_hits);
    println!("allow_bin={}", summary.allow_bin);
    println!("allow_test={}", summary.allow_test);
    println!("allow_float_ingest={}", summary.allow_float_ingest);
    println!("warn_other={}", summary.warn_other);
    println!("core_leak={}", summary.core_leak);

    if summary.core_leak > 0 {
        std::process::exit(1);
    }
}

fn collect_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    let mut paths: Vec<PathBuf> = entries.filter_map(|e| e.ok().map(|d| d.path())).collect();
    paths.sort();
    for path in paths {
        if path.is_dir() {
            if has_component(&path, "target") || has_component(&path, "examples") {
                continue;
            }
            collect_rs_files(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

fn analyze_file(file: &Path, ast: &File, out: &mut Vec<Hit>) {
    let file_gate = gate_from_attrs(&ast.attrs);
    let rel = display_rel(file);
    let mut visitor = TypeHitVisitor {
        file: rel,
        file_gate,
        current_gate: 0,
        hits: Vec::new(),
    };
    visitor.visit_file(ast);
    out.extend(visitor.hits);
}

struct TypeHitVisitor {
    file: String,
    file_gate: u8,
    current_gate: u8,
    hits: Vec<Hit>,
}

impl<'ast> Visit<'ast> for TypeHitVisitor {
    fn visit_item(&mut self, i: &'ast Item) {
        let prior = self.current_gate;
        self.current_gate = prior | gate_from_attrs(item_attrs(i));
        syn::visit::visit_item(self, i);
        self.current_gate = prior;
    }

    fn visit_impl_item(&mut self, i: &'ast ImplItem) {
        let prior = self.current_gate;
        self.current_gate = prior | gate_from_attrs(impl_item_attrs(i));
        syn::visit::visit_impl_item(self, i);
        self.current_gate = prior;
    }

    fn visit_trait_item(&mut self, i: &'ast TraitItem) {
        let prior = self.current_gate;
        self.current_gate = prior | gate_from_attrs(trait_item_attrs(i));
        syn::visit::visit_trait_item(self, i);
        self.current_gate = prior;
    }

    fn visit_foreign_item(&mut self, i: &'ast ForeignItem) {
        let prior = self.current_gate;
        self.current_gate = prior | gate_from_attrs(foreign_item_attrs(i));
        syn::visit::visit_foreign_item(self, i);
        self.current_gate = prior;
    }

    fn visit_type_path(&mut self, t: &'ast TypePath) {
        if let Some((span, ty)) = float_segment_in_path(&t.path) {
            let line = span_line(span);
            self.hits.push(Hit {
                file: self.file.clone(),
                line,
                ty,
                gate: self.file_gate | self.current_gate,
            });
        }
        syn::visit::visit_type_path(self, t);
    }

    fn visit_expr_path(&mut self, p: &'ast ExprPath) {
        if let Some((span, ty)) = float_segment_in_path(&p.path) {
            let line = span_line(span);
            self.hits.push(Hit {
                file: self.file.clone(),
                line,
                ty,
                gate: self.file_gate | self.current_gate,
            });
        }
        syn::visit::visit_expr_path(self, p);
    }

    fn visit_lit_float(&mut self, lit: &'ast syn::LitFloat) {
        let suffix = lit.suffix();
        if suffix == "f32" || suffix == "f64" {
            self.hits.push(Hit {
                file: self.file.clone(),
                line: span_line(lit.span()),
                ty: if suffix == "f32" { "f32" } else { "f64" },
                gate: self.file_gate | self.current_gate,
            });
        }
        syn::visit::visit_lit_float(self, lit);
    }
}

fn float_segment_in_path(path: &SynPath) -> Option<(Span, &'static str)> {
    for seg in &path.segments {
        if seg.ident == "f32" {
            return Some((seg.ident.span(), "f32"));
        }
        if seg.ident == "f64" {
            return Some((seg.ident.span(), "f64"));
        }
    }
    None
}

fn span_line(span: Span) -> usize {
    let line = span.start().line;
    if line == 0 {
        1
    } else {
        line
    }
}

fn gate_from_attrs(attrs: &[Attribute]) -> u8 {
    let mut out = 0u8;
    for attr in attrs {
        if !attr.path().is_ident("cfg") {
            continue;
        }
        let meta = &attr.meta;
        let tokens = match meta {
            Meta::List(m) => m.tokens.clone(),
            _ => continue,
        };
        let expr = match syn::parse2::<Expr>(tokens) {
            Ok(e) => e,
            Err(_) => continue,
        };
        if cfg_expr_has_test(&expr) {
            out |= GATE_TEST;
        }
        if cfg_expr_has_float_ingest(&expr) {
            out |= GATE_FLOAT_INGEST;
        }
    }
    out
}

fn cfg_expr_has_test(expr: &Expr) -> bool {
    match expr {
        Expr::Path(ExprPath { path, .. }) => path.is_ident("test"),
        Expr::Paren(ExprParen { expr, .. }) => cfg_expr_has_test(expr),
        Expr::Group(ExprGroup { expr, .. }) => cfg_expr_has_test(expr),
        Expr::Unary(ExprUnary { expr, .. }) => cfg_expr_has_test(expr),
        Expr::Binary(ExprBinary { left, right, .. }) => {
            cfg_expr_has_test(left) || cfg_expr_has_test(right)
        }
        Expr::Assign(ExprAssign { left, right, .. }) => {
            cfg_expr_has_test(left) || cfg_expr_has_test(right)
        }
        Expr::Call(ExprCall { func, args, .. }) => {
            cfg_expr_has_test(func) || args.iter().any(cfg_expr_has_test)
        }
        _ => false,
    }
}

fn cfg_expr_has_float_ingest(expr: &Expr) -> bool {
    match expr {
        Expr::Path(_) => false,
        Expr::Assign(ExprAssign { left, right, .. }) => {
            is_feature_eq_float_ingest(left, right)
                || cfg_expr_has_float_ingest(left)
                || cfg_expr_has_float_ingest(right)
        }
        Expr::Paren(ExprParen { expr, .. }) => cfg_expr_has_float_ingest(expr),
        Expr::Group(ExprGroup { expr, .. }) => cfg_expr_has_float_ingest(expr),
        Expr::Unary(ExprUnary { expr, .. }) => cfg_expr_has_float_ingest(expr),
        Expr::Binary(ExprBinary { left, right, .. }) => {
            cfg_expr_has_float_ingest(left) || cfg_expr_has_float_ingest(right)
        }
        Expr::Call(ExprCall { func, args, .. }) => {
            cfg_expr_has_float_ingest(func) || args.iter().any(cfg_expr_has_float_ingest)
        }
        _ => false,
    }
}

fn is_feature_eq_float_ingest(left: &Expr, right: &Expr) -> bool {
    let left_is_feature = matches!(
        left,
        Expr::Path(ExprPath { path, .. }) if path.is_ident("feature")
    );
    let right_is_float_ingest = matches!(
        right,
        Expr::Lit(expr_lit) if matches!(&expr_lit.lit, Lit::Str(s) if s.value() == "float-ingest")
    );
    left_is_feature && right_is_float_ingest
}

fn item_attrs(i: &Item) -> &[Attribute] {
    match i {
        Item::Const(x) => &x.attrs,
        Item::Enum(x) => &x.attrs,
        Item::ExternCrate(x) => &x.attrs,
        Item::Fn(x) => &x.attrs,
        Item::ForeignMod(x) => &x.attrs,
        Item::Impl(x) => &x.attrs,
        Item::Macro(x) => &x.attrs,
        Item::Mod(x) => &x.attrs,
        Item::Static(x) => &x.attrs,
        Item::Struct(x) => &x.attrs,
        Item::Trait(x) => &x.attrs,
        Item::TraitAlias(x) => &x.attrs,
        Item::Type(x) => &x.attrs,
        Item::Union(x) => &x.attrs,
        Item::Use(x) => &x.attrs,
        _ => &[],
    }
}

fn impl_item_attrs(i: &ImplItem) -> &[Attribute] {
    match i {
        ImplItem::Const(x) => &x.attrs,
        ImplItem::Fn(x) => &x.attrs,
        ImplItem::Macro(x) => &x.attrs,
        ImplItem::Type(x) => &x.attrs,
        _ => &[],
    }
}

fn trait_item_attrs(i: &TraitItem) -> &[Attribute] {
    match i {
        TraitItem::Const(x) => &x.attrs,
        TraitItem::Fn(x) => &x.attrs,
        TraitItem::Macro(x) => &x.attrs,
        TraitItem::Type(x) => &x.attrs,
        _ => &[],
    }
}

fn foreign_item_attrs(i: &ForeignItem) -> &[Attribute] {
    match i {
        ForeignItem::Fn(x) => &x.attrs,
        ForeignItem::Macro(x) => &x.attrs,
        ForeignItem::Static(x) => &x.attrs,
        ForeignItem::Type(x) => &x.attrs,
        _ => &[],
    }
}

fn display_rel(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn has_component(path: &Path, name: &str) -> bool {
    path.components()
        .any(|c| matches!(c, Component::Normal(x) if x == name))
}

fn is_bin_path(path: &str) -> bool {
    let p = Path::new(path);
    let comps: Vec<String> = p
        .components()
        .filter_map(|c| match c {
            Component::Normal(s) => Some(s.to_string_lossy().to_string()),
            _ => None,
        })
        .collect();
    comps.windows(2).any(|w| w[0] == "src" && w[1] == "bin")
}

fn is_test_path(path: &str) -> bool {
    let p = Path::new(path);
    p.components().any(|c| {
        matches!(
            c,
            Component::Normal(s)
                if s == "tests" || s == "benches"
        )
    })
}

fn is_core_path(path: &str) -> bool {
    (path.starts_with("crates/dpw4/src/") && !path.starts_with("crates/dpw4/src/bin/"))
        || path.starts_with("crates/replay-core/src/")
        || path.starts_with("crates/replay-embed/src/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    fn gate_from_attr(attr: Attribute) -> u8 {
        gate_from_attrs(&[attr])
    }

    #[test]
    fn detects_true_cfg_cases() {
        let cases: [Attribute; 7] = [
            parse_quote!(#[cfg(test)]),
            parse_quote!(#[cfg(any(test, unix))]),
            parse_quote!(#[cfg(all(test, feature = "float-ingest"))]),
            parse_quote!(#[cfg(feature = "float-ingest")]),
            parse_quote!(#[cfg(any(feature = "float-ingest", feature = "other"))]),
            parse_quote!(#[cfg(not(feature = "float-ingest"))]),
            parse_quote!(#[cfg((feature = "float-ingest"))]),
        ];

        for attr in cases {
            let gate = gate_from_attr(attr);
            assert!((gate & GATE_TEST) != 0 || (gate & GATE_FLOAT_INGEST) != 0);
        }
    }

    #[test]
    fn detects_expected_false_cases() {
        let not_float: [Attribute; 4] = [
            parse_quote!(#[cfg(feature = "float_ingest")]),
            parse_quote!(#[cfg(feature = "float-ingest ")]),
            parse_quote!(#[cfg(feature = "FLOAT-INGEST")]),
            parse_quote!(#[cfg(any(feature = "other", unix))]),
        ];
        for attr in not_float {
            let gate = gate_from_attr(attr);
            assert_eq!(gate & GATE_FLOAT_INGEST, 0);
        }

        let not_test: Attribute = parse_quote!(#[cfg(tests)]);
        let gate = gate_from_attr(not_test);
        assert_eq!(gate & GATE_TEST, 0);
    }
}
