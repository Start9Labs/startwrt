use darling::FromDeriveInput;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{parse_macro_input, parse_quote, Data, DeriveInput, GenericArgument, Ident, Path, Type};

#[derive(FromDeriveInput, Default)]
#[darling(default, attributes(uci))]
struct UciSectionOpts {
    ty: Option<String>,
}

struct UciField {
    placehold: Ident,
    field: Ident,
    name: String,
    is_opt: bool,
    is_vec: bool,
    crat: Path,
}

impl UciField {
    fn read_decl(&self) -> TokenStream {
        let UciField { placehold, .. } = self;
        if self.is_vec {
            quote!(let mut #placehold = Vec::new();)
        } else {
            quote!(let mut #placehold = None;)
        }
    }

    fn read_option_arm(&self) -> TokenStream {
        if self.is_vec {
            return TokenStream::new();
        }
        let UciField {
            placehold, name, ..
        } = self;
        quote! {
            #name if #placehold.is_none() => #placehold = Some(std::str::FromStr::from_str(&value.as_str())?),
        }
    }

    fn read_list_arm(&self) -> TokenStream {
        if !self.is_vec {
            return TokenStream::new();
        }
        let UciField {
            placehold, name, ..
        } = self;
        quote! {
            #name => #placehold.push(std::str::FromStr::from_str(&item.as_str())?),
        }
    }

    fn read_init(&self) -> TokenStream {
        let UciField {
            placehold,
            field,
            crat,
            ..
        } = self;
        if self.is_opt || self.is_vec {
            quote! { #field: #placehold, }
        } else {
            let msg = format!("missing field {}", field);
            quote! { #field: #placehold.ok_or(#crat::error!(#msg))?, }
        }
    }

    fn write_decl(&self) -> TokenStream {
        let UciField {
            placehold,
            field,
            crat,
            name,
            ..
        } = self;
        match (self.is_opt, self.is_vec) {
            (false, false) => quote! {
                let mut #placehold = Some(#crat::Line::Option {
                    option: #crat::Token::from_str(#name, arena),
                    value: #crat::Token::from_display(&self.#field, arena),
                })
                .into_iter();
            },
            (true, false) => quote! {
                let mut #placehold = self.#field.iter().map(|value| #crat::Line::Option {
                    option: #crat::Token::from_str(#name, arena),
                    value: #crat::Token::from_display(value, arena),
                });
            },
            (false, true) => quote! {
                let mut #placehold = self.#field.iter().map(|item| #crat::Line::List {
                    list: #crat::Token::from_str(#name, arena),
                    item: #crat::Token::from_display(item, arena),
                });
            },
            _ => panic!("can not be both Option and Vec"),
        }
    }

    fn write_option_arm(&self) -> TokenStream {
        if self.is_vec {
            return TokenStream::new();
        }
        let UciField {
            placehold, name, ..
        } = self;
        quote! {
            #name => #placehold.next(),
        }
    }

    fn write_list_arm(&self) -> TokenStream {
        if !self.is_vec {
            return TokenStream::new();
        }
        let UciField {
            placehold, name, ..
        } = self;
        quote! {
            #name => #placehold.next(),
        }
    }
}

fn chained_write_iters(fields: &[UciField]) -> Option<TokenStream> {
    let mut chained = None;
    for UciField { placehold, .. } in fields.iter().rev() {
        chained = Some(match chained {
            Some(joined) => quote! { #placehold.chain(#joined) },
            None => quote! { #placehold },
        });
    }
    chained
}

fn read_body(fields: &[UciField], struc: Ident, _ty: String, crat: Path) -> TokenStream {
    let decl = fields.iter().map(UciField::read_decl);
    let option_arm = fields.iter().map(UciField::read_option_arm);
    let list_arm = fields.iter().map(UciField::read_list_arm);
    let init = fields.iter().map(UciField::read_init);
    quote! {
        let Some(#crat::Line::Section { .. }) = lines.get(index) else {
            bail!("line {index} does not start a section")
        };
        #(#decl)*

        loop {
            index += 1;
            match lines.get(index) {
                Some(#crat::Line::Option { option, value }) => match &*option.as_str() {
                    #(#option_arm)*
                    _ => continue,
                },
                Some(#crat::Line::List { list, item }) => match &*list.as_str() {
                    #(#list_arm)*
                    _ => continue,
                },
                None | Some(#crat::Line::Section { .. }) => break,
                _ => continue,
            }
        }

        Ok(#struc {
            #(#init)*
        })
    }
}

fn write_body(fields: &[UciField], _struc: Ident, ty: String, crat: Path) -> TokenStream {
    let decl = fields.iter().map(UciField::write_decl);
    let option_arm = fields.iter().map(UciField::write_option_arm);
    let list_arm = fields.iter().map(UciField::write_list_arm);
    let chain = chained_write_iters(fields);
    let not_section_err = format!("line {{index}} is not a {ty} section");
    quote! {
        let Some(#crat::Line::Section { ty, .. }) = lines.get(index) else {
            bail!("line {index} does not start a section")
        };
        if ty.as_str() != #ty {
            bail!(#not_section_err)
        }

        #(#decl)*

        let mut insert_after = index;
        loop {
            index += 1;
            let Some(line) = lines.get_mut(index) else {
                break;
            };
            if line.is_in_section() {
                insert_after = index;
            }
            *line = match line {
                #crat::Line::Option { option, .. } => match &*option.as_str() {
                    #(#option_arm)*
                    _ => continue,
                },
                #crat::Line::List { list, .. } => match &*list.as_str() {
                    #(#list_arm)*
                    _ => continue,
                },
                #crat::Line::Section { .. } => break,
                _ => continue,
            }
            .unwrap_or(#crat::Line::Skip);
            insert_after = index;
        }

        lines.splice(
            insert_after+1..insert_after+1,
            #chain,
        );

        Ok(())
    }
}

fn append_body(fields: &[UciField], _struc: Ident, ty: String, crat: Path) -> TokenStream {
    let decl = fields.iter().map(UciField::write_decl);
    let chain = chained_write_iters(fields);
    quote! {
        #(#decl)*

        if !lines.is_empty() {
            lines.push(#crat::Line::Empty);
        }

        lines.push(#crat::Line::Section {
            ty: #crat::Token::from_str(#ty, arena),
            name: name.map(|n| #crat::Token::from_str(n, arena)),
        });
        lines.extend(#chain);

        Ok(())
    }
}

fn is_collection_with_generic(ty: &Type, collection: &str) -> bool {
    if let Type::Path(path) = ty {
        if let Some(segment) = path.path.segments.first() {
            if segment.ident == collection {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if args.args.len() == 1 {
                        if let GenericArgument::Type(_) = args.args[0] {
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
}

#[proc_macro_derive(UciSection, attributes(uci))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let opts = UciSectionOpts::from_derive_input(&input).expect("Wrong options");

    let crat: Path = parse_quote! { ::uciedit };
    let struc = input.ident;
    let ty = opts.ty.unwrap_or(struc.to_string().to_lowercase());
    let Data::Struct(struct_data) = input.data else {
        panic!("only structs are supported")
    };
    let fields: Vec<_> = struct_data
        .fields
        .into_iter()
        .map(|f| {
            let i = f.ident.unwrap();
            UciField {
                placehold: format_ident!("field_{}", i),
                field: i.clone(),
                name: i.to_string(),
                is_opt: is_collection_with_generic(&f.ty, "Option"),
                is_vec: is_collection_with_generic(&f.ty, "Vec"),
                crat: crat.clone(),
            }
        })
        .collect();

    let read_body = read_body(&fields, struc.clone(), ty.clone(), crat.clone());
    let write_body = write_body(&fields, struc.clone(), ty.clone(), crat.clone());
    let append_body = append_body(&fields, struc.clone(), ty.clone(), crat.clone());

    let (_, type_generics, where_clause) = input.generics.split_for_impl();
    let mut lt_generics = input.generics.clone();
    lt_generics.params.push(parse_quote! { 'a });
    let (impl_generics, _, _) = lt_generics.split_for_impl();

    quote! {
        impl #impl_generics #crat::UciSection<'a> for #struc #type_generics #where_clause {
            fn read(lines: &#crat::Lines<'a>, mut index: usize) -> Result<Self, #crat::Error> {
                #read_body
            }

            #[allow(unused_mut)]
            fn write(
                &self,
                lines: &mut #crat::Lines<'a>,
                arena: &'a #crat::Arena,
                mut index: usize,
            ) -> Result<(), #crat::Error> {
                #write_body
            }

            #[allow(unused_mut)]
            fn append(
                &self,
                lines: &mut #crat::Lines<'a>,
                arena: &'a #crat::Arena,
                name: Option<&'a str>,
            ) -> Result<(), #crat::Error> {
                #append_body
            }
        }

    }
    .into_token_stream()
    .into()
}
