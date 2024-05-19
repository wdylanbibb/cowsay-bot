use proc_macro::TokenStream;

/// ```rust
/// cmd_enum! {
///     #attributes
///     #visibility enum #name(#cmd) {
///         #cmd_attr
///         #cmd_name
///     }
/// }
/// ```
struct CmdEnum {
    attributes: Vec<syn::Attribute>,
    visibility: syn::Visibility,
    name: syn::Ident,
    cmd: syn::LitStr,
    // cmd_attr: Vec<syn::Attribute>,
    cmd_name: syn::LitStr,
}

impl syn::parse::Parse for CmdEnum {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content_paren;
        let content_braced;
        let attributes = input.call(syn::Attribute::parse_outer)?;
        let visibility = input.parse()?;
        input.parse::<syn::Token![enum]>()?;
        let name = input.parse()?;
        syn::parenthesized!(content_paren in input);
        let cmd = content_paren.parse()?;
        syn::braced!(content_braced in input);
        // let cmd_attr = content_braced.call(syn::Attribute::parse_outer)?;
        let cmd_name = content_braced.parse()?;
        Ok(Self {
            attributes,
            visibility,
            name,
            cmd,
            // cmd_attr,
            cmd_name,
        })
    }
}

#[proc_macro]
pub fn cmd_enum(input: TokenStream) -> TokenStream {
    let CmdEnum {
        attributes,
        visibility,
        name,
        cmd,
        // cmd_attr,
        cmd_name,
    } = syn::parse(input).unwrap();

    // Single string of the output of `cmd`
    let cmd = String::from_utf8(
        std::process::Command::new("sh")
            .arg("-c")
            .arg(cmd.value())
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();

    let cmd_vec = cmd
        .trim()
        .split('\n')
        .map(|cow| {
            let ret = String::from_utf8(
                std::process::Command::new("sh")
                    .arg("-c")
                    .arg(cmd_name.value().replace("$1", cow))
                    .output()
                    .unwrap()
                    .stdout,
            )
            .unwrap();
            syn::parse_str::<syn::Ident>(ret.trim()).unwrap()
        })
        .collect::<Vec<_>>();

    let cmd_name_vec = cmd.trim().split('\n').collect::<Vec<_>>();

    quote::quote! {
        #(#attributes)*
        #visibility enum #name {
            #(
                #cmd_vec
            ),*
        }

        impl #name {
            fn name(&self) -> String {
                match self {
                    #(
                        Self::#cmd_vec => String::from(#cmd_name_vec)
                    ),*
                }
            }
        }
    }
    .into()
}
