#[derive(Clone, Debug, Default)]
pub struct Hooks {
    pub registered: Vec<syn::Expr>,
}

impl syn::parse::Parse for Hooks {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut hooks = vec![];

        while !input.is_empty() {
            let expr: syn::Expr = input.parse()?;

            match expr {
                syn::Expr::Path(_) | syn::Expr::Struct(_) => {
                    hooks.push(expr);
                }
                _ => {
                    return Err(syn::Error::new_spanned(
                        expr,
                        "`#[hooks(..)]` only supports paths or struct literals",
                    ))
                }
            }

            if input.peek(syn::Token![,]) {
                input.parse::<syn::Token![,]>()?;
            }
        }

        Ok(Self { registered: hooks })
    }
}
