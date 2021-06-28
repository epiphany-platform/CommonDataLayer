use proc_macro_error::{abort, ResultExt};

pub enum Attribute {
    KeyValue { path: syn::Path, lit: syn::Lit },
    Toggle { path: syn::Path },
}

impl Attribute {
    pub fn toggled(&self, cond: &syn::Path) -> bool {
        match self {
            Attribute::Toggle { path } => path == cond,
            _ => false,
        }
    }

    pub fn parse_if<T: syn::parse::Parse>(&self, cond: &syn::Path) -> Option<T> {
        match self {
            Attribute::KeyValue { path, .. } => {
                if path == cond {
                    Some(self.parse())
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn parse<T: syn::parse::Parse>(&self) -> T {
        match &self {
            Attribute::KeyValue {
                path,
                lit: syn::Lit::Str(lit_str),
            } => lit_str
                .parse()
                .expect_or_abort(&format!("Expected {:?} to be path", path)),
            Attribute::KeyValue { path, lit } => {
                abort!(lit, "expected #[rpc({:?} = \"...\")]", path)
            }
            Attribute::Toggle { path } => {
                abort!(path, "expected attribute in format #[rpc(key = \"...\")]")
            }
        }
    }
}

pub(crate) fn get<'a>(attrs: impl Iterator<Item = &'a syn::Attribute>) -> Vec<Attribute> {
    let rpc_path = syn::parse_quote!(rpc);

    attrs
        .filter_map(|attr| attr.parse_meta().ok())
        .filter_map(|meta| match meta {
            syn::Meta::List(list) => Some(list),
            _ => None,
        })
        .filter(|meta| meta.path == rpc_path)
        .flat_map(|meta| meta.nested)
        .filter_map(|nested| match nested {
            syn::NestedMeta::Meta(meta) => Some(meta),
            _ => None,
        })
        .filter_map(|meta| match meta {
            syn::Meta::NameValue(kv) => Some(Attribute::KeyValue {
                path: kv.path,
                lit: kv.lit,
            }),
            syn::Meta::Path(p) => Some(Attribute::Toggle { path: p }),
            _ => None,
        })
        .collect()
}
