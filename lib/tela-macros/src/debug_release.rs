use proc_macro_error::abort;
use syn::{
    parse::{Parse, ParseStream},
    Expr, Ident, Token,
};

pub struct DebugRelease(pub Expr, pub Expr);

macro_rules! checked {
    (
        ($first: ident, $fval: ident),
        ($second: ident, $sval: ident),
        @debug [$debug: literal, $($doption: literal),*],
        @release [$release: literal, $($roption: literal),*] $(,)?
    ) => {
        {
            if $first.to_string() == $second.to_string() {
                match $first.to_string().as_str() {
                    $debug $(| $doption)* => abort!($second, "Expected release tag"),
                    $release $(| $roption)* => abort!($second, "Expected debug tag"),
                    _ => abort!($first, "Unkown tag name"; help="Expected debug or release tags")
                }
            }

            let tag = $second.to_string();
            let tag = tag.as_str();

            match $first.to_string().as_str() {
                $debug $(| $doption)* => {
                    if tag != $release $(&& tag != $roption)* {
                        abort!($second, "Expected release tag")
                    }
                    Ok(DebugRelease($fval, $sval))
                },
                $release $(| $roption)* => {
                    if tag != $debug $(&& tag != $doption)* {
                        abort!($second, "Expected debug tag")
                    }
                    Ok(DebugRelease($sval, $fval))
                }
                _ => abort!($first, "Unkown tag name"; help="Expected debug or release tags")
            }



        }
    };
}

impl Parse for DebugRelease {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let tag1 = input.parse::<Ident>()?;
        let _ = input.parse::<Token![:]>();
        let val1 = input.parse::<Expr>()?;

        let _ = input.parse::<Token![,]>();

        let tag2 = input.parse::<Ident>()?;
        let _ = input.parse::<Token![:]>();
        let val2 = input.parse::<Expr>()?;

        let _ = input.parse::<Token![,]>();

        checked!(
            (tag1, val1),
            (tag2, val2),
            @debug ["debug", "dbg", "d"],
            @release ["release", "rls", "r"]
        )
    }
}
