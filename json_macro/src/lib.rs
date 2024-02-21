use proc_macro::TokenStream;
use quote::quote;
use syn::{Attribute, Data, DeriveInput, Expr, Field, GenericArgument, Ident, Lit, PathArguments, Type};

#[proc_macro_derive(JsonParse, attributes(json))]
pub fn derive_json(_item: TokenStream) -> TokenStream {
  let tree: DeriveInput = syn::parse(_item).unwrap();
  let st = match tree.data {
    Data::Struct(s) => s,
    _ => panic!("JsonParse can only be derived for structs"),
  };

  let DeriveInput { ident: struct_name, .. } = tree;
  let field_count = st.fields.iter().filter(|Field { ident, .. }| ident.is_some()).count();
  let mut bitflag = 0;
  let arms = st.fields.iter().filter_map(|field| {
    let Field { ident, ty, attrs, .. } = field.clone();
    let field_name = ident?;
    let arm = find_alias(&attrs, &field_name);

    let method = match find_type(&ty) {
      JsonType::Custom => quote!(<#ty as json::JsonParser>::parse_json(value)?),
      JsonType::Array => quote!(Vec::parse_json(value)?),
      JsonType::Option => quote!(Option::parse_json(value)?),
      JsonType::Bool => quote!(value.bool(ln, col)?),
      JsonType::Double => quote!(value.double(ln, col)? as #ty),
      JsonType::Integer => quote!(value.int(ln, col)? as #ty),
      JsonType::String => quote!(value.string(ln, col)?),
    };

    bitflag = if bitflag == 0 { 1 } else { bitflag << 1 };
    Some(quote! {
      #arm => {
        #field_name = Some(#method);
        flags &= !#bitflag;
      }
    })
  });

  let vars = st.fields.iter().filter_map(|field| {
    let Field { ident, .. } = field.clone();
    let field_name = ident?;
    Some(quote! {
      let mut #field_name = None;
    })
  });

  let initialized_fields = st.fields.iter().filter_map(|field| {
    let Field { ident, attrs, .. } = field.clone();
    let field_name = ident?;
    let alias = find_alias(&attrs, &field_name);
    Some(quote! {
      #field_name: #field_name.ok_or(json::JsonError::MissingField(obj.ln, obj.col, #alias))?,
    })
  });

  let mut bits = 1;
  for _ in 1..field_count {
    bits <<= 1;
    bits += 1;
  }

  quote! {
    impl json::JsonParser for #struct_name {
      fn parse_json(json: json::JsonValue) -> json::JsonResult<Self> {
        if let json::JsonValue::Object(mut obj) = json {
          #(#vars)*
          let mut flags = #bits;

          while let Some(res) = json::LendingIterator::next(&mut obj) {
            let res = res?;
            let (json::JsonEntry(key, value), ln, col) = res;
            match key {
              #(#arms)*
              _ => (),
            }

            if flags == 0 {
              break;
            }
          }

          return Ok(#struct_name {
            #(#initialized_fields)*
          });
        }

        Err(json::JsonError::NoMatch)
      }
    }
  }
  .into()
}

fn find_alias(attrs: &[Attribute], field_name: &Ident) -> String {
  let alias = attrs.iter().find_map(|attr| {
    let expr: Expr = attr.parse_args().unwrap();
    match expr {
      Expr::Assign(a) => {
        let Expr::Path(left) = *a.left else { return None };
        if !left.path.is_ident("alias") {
          return None;
        }
        let Expr::Lit(arm) = *a.right else { return None };
        let Lit::Str(arm) = arm.lit else { return None };
        Some(arm.value())
      }
      _ => None,
    }
  });

  alias.unwrap_or_else(|| format!("{field_name}"))
}

fn find_type(ty: &Type) -> JsonType {
  match &ty {
    Type::Path(p) => {
      let ident = &p.path.segments.last().unwrap().ident;
      match ident.to_string().as_str() {
        "Box" => {
          let PathArguments::AngleBracketed(ref a) = p.path.segments.last().unwrap().arguments else {
            return JsonType::Custom;
          };
          let GenericArgument::Type(ty) = a.args.last().unwrap() else {
            return JsonType::Custom;
          };
          let Type::Path(p) = ty else {
            return JsonType::Custom;
          };
          if p.path.segments.last().unwrap().ident != "str" {
            return JsonType::Custom;
          }
          JsonType::String
        }
        "Vec" => JsonType::Array,
        "isize" | "usize" | "i128" | "u128" | "i64" | "u64" | "i32" | "u32" | "i16" | "u16" | "i8" | "u8" => JsonType::Integer,
        "f64" | "f32" => JsonType::Double,
        "bool" => JsonType::Bool,
        "Option" => JsonType::Option,
        _ => JsonType::Custom,
      }
    }
    _ => JsonType::Custom,
  }
}

enum JsonType {
  String,
  Integer,
  Double,
  Bool,
  Array,
  Option,
  Custom,
}
