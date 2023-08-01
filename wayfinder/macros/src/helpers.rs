use proc_macro_error::abort;
use syn::{GenericArgument, PathArguments, Type};

pub fn path_of(path: &Type, ty: &str) -> bool {
    get_path_name(path).as_str() == ty
}

pub fn get_path_generic(path: &Type) -> Type {
    if let Type::Path(pt) = path {
        let last = pt.path.segments.last().unwrap();
        match &last.arguments {
            PathArguments::AngleBracketed(abga) => match abga.args.first().unwrap() {
                GenericArgument::Type(ty) => ty.clone(),
                _ => abort!(abga, "Expected generic type"),
            },
            _ => abort!(last, "Expected type with generic"),
        }
    } else {
        abort!(path, "[Internal:get_path_generic] Expected a Path type")
    }
}

pub fn get_path_name(path: &Type) -> String {
    if let Type::Path(pt) = path {
        pt.path.segments.last().unwrap().ident.to_string()
    } else {
        String::new()
    }
}
