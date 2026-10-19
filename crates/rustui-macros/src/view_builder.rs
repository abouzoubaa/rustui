use proc_macro::TokenStream;
use proc_macro2::{Delimiter, TokenStream as TokenStream2, TokenTree};
use quote::quote;
use syn::{
    Expr, Result, Token, braced,
    parse::{Parse, ParseStream},
    parse_quote,
    visit_mut::VisitMut,
};

struct ShorthandButtonAction;

impl VisitMut for ShorthandButtonAction {
    fn visit_expr_call_mut(&mut self, call: &mut syn::ExprCall) {
        syn::visit_mut::visit_expr_call_mut(self, call);

        let Expr::Path(function) = &*call.func else {
            return;
        };
        if function.qself.is_some()
            || function.path.segments.len() != 1
            || function.path.segments[0].ident != "Button"
        {
            return;
        }

        if call.args.len() != 2 {
            return;
        }
        let Expr::Path(handler) = &call.args[1] else {
            return;
        };
        if handler.qself.is_some() || handler.path.segments.len() != 1 {
            return;
        }

        let method = handler.path.segments[0].ident.clone();
        call.args[1] = parse_quote!(self.action(Self::#method));
    }
}

pub fn expand(input: TokenStream) -> Result<TokenStream> {
    expand_with(input.into(), true).map(Into::into)
}

pub fn expand_horizontal(input: TokenStream) -> Result<TokenStream> {
    expand_with(input.into(), false).map(Into::into)
}

struct StackChildren(Vec<StackChild>);

enum StackChild {
    View(Expr),
    Conditional(Conditional),
}

struct Conditional {
    condition: Expr,
    then_children: StackChildren,
    otherwise: Option<Otherwise>,
}

enum Otherwise {
    Children(StackChildren),
    Conditional(Box<Conditional>),
}

impl Parse for StackChildren {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut children = Vec::new();
        while !input.is_empty() {
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
                continue;
            }
            if input.peek(Token![;]) {
                input.parse::<Token![;]>()?;
                continue;
            }
            if input.peek(Token![if]) {
                children.push(StackChild::Conditional(input.parse()?));
            } else {
                if input.peek(Token![for]) || input.peek(Token![match]) {
                    return Err(input.error("stack macros do not support `for` or `match`"));
                }
                let mut child = parse_view_child(input)?;
                if !is_constructor(&child) {
                    return Err(syn::Error::new_spanned(
                        child,
                        "stack children must be constructor calls, nested stack calls, or `if` branches",
                    ));
                }
                ShorthandButtonAction.visit_expr_mut(&mut child);
                children.push(StackChild::View(child));
            }
        }
        Ok(Self(children))
    }
}

fn parse_view_child(input: ParseStream<'_>) -> Result<Expr> {
    // Preserve the DSL's adjacent-child boundary: a parenthesized next child
    // must not become another call on the preceding view, as a Rust Expr parser
    // would interpret `controls.spacing(8.0) (condition).then(...)`.
    let mut tokens = TokenStream2::new();
    while !input.is_empty() {
        let token: TokenTree = input.parse()?;
        let closes_constructor = matches!(
            &token,
            TokenTree::Group(group)
                if matches!(group.delimiter(), Delimiter::Parenthesis | Delimiter::Brace)
        );
        tokens.extend([token]);
        if closes_constructor {
            break;
        }
    }
    while has_method_call(input) {
        for _ in 0..3 {
            tokens.extend([input.parse::<TokenTree>()?]);
        }
    }
    syn::parse2(tokens)
}

fn has_method_call(input: ParseStream<'_>) -> bool {
    let lookahead = input.fork();
    matches!(lookahead.parse::<TokenTree>(), Ok(TokenTree::Punct(dot)) if dot.as_char() == '.')
        && matches!(lookahead.parse::<TokenTree>(), Ok(TokenTree::Ident(_)))
        && matches!(
            lookahead.parse::<TokenTree>(),
            Ok(TokenTree::Group(arguments)) if arguments.delimiter() == Delimiter::Parenthesis
        )
}

fn is_constructor(expression: &Expr) -> bool {
    match expression {
        Expr::Call(_) | Expr::MethodCall(_) | Expr::Macro(_) | Expr::Struct(_) => true,
        _ => false,
    }
}

impl Parse for Conditional {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        input.parse::<Token![if]>()?;
        let condition = Expr::parse_without_eager_brace(input)?;
        let then_body;
        braced!(then_body in input);
        let then_children = then_body.parse()?;
        let otherwise = if input.peek(Token![else]) {
            input.parse::<Token![else]>()?;
            Some(if input.peek(Token![if]) {
                Otherwise::Conditional(Box::new(input.parse()?))
            } else {
                let else_body;
                braced!(else_body in input);
                Otherwise::Children(else_body.parse()?)
            })
        } else {
            None
        };
        Ok(Self {
            condition,
            then_children,
            otherwise,
        })
    }
}

impl StackChildren {
    fn append_to(&self, mut builder: TokenStream2) -> TokenStream2 {
        for child in &self.0 {
            builder = match child {
                StackChild::View(view) => quote!(#builder.child(#view)),
                StackChild::Conditional(conditional) => {
                    let group = conditional.group();
                    quote!(#builder.group(#group))
                }
            };
        }
        builder
    }

    fn group(&self) -> TokenStream2 {
        self.append_to(quote!(::rustui::layout::Group::new()))
    }
}

impl Conditional {
    fn group(&self) -> TokenStream2 {
        let condition = &self.condition;
        let then_group = self.then_children.group();
        let else_group = match &self.otherwise {
            Some(Otherwise::Children(children)) => children.group(),
            Some(Otherwise::Conditional(conditional)) => conditional.group(),
            None => quote!(::rustui::layout::Group::new()),
        };
        quote!(if #condition { #then_group } else { #else_group })
    }
}

fn expand_with(input: TokenStream2, vertical: bool) -> Result<TokenStream2> {
    let children: StackChildren = syn::parse2(input)?;
    let mut stack = if vertical {
        quote!(::rustui::layout::VStack::new())
    } else {
        quote!(::rustui::layout::HStack::new())
    };
    stack = children.append_to(stack);
    Ok(stack)
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::ToTokens as _;

    fn expanded(input: TokenStream2, vertical: bool) -> TokenStream2 {
        let result = expand_with(input, vertical).expect("valid stack grammar");
        syn::parse2::<Expr>(result.clone()).expect("expansion is a Rust expression");
        result
    }

    fn assert_same_expression(actual: TokenStream2, expected: TokenStream2) {
        // Syn emits closure pipes as separate punctuations (`| |`), whereas
        // quote preserves a source `||` token's joint spacing. Compare both
        // expressions after the same AST roundtrip, retaining every AST node.
        let actual: Expr = syn::parse2(actual).expect("actual expression");
        let expected: Expr = syn::parse2(expected).expect("expected expression");
        assert_eq!(
            actual.into_token_stream().to_string(),
            expected.into_token_stream().to_string()
        );
    }

    #[test]
    fn preserves_adjacent_constructors_separators_and_chained_builders() {
        let actual = expanded(
            quote! {
                Text("first").size(14.0)
                Button("second", hit).id("button"),;
                HStack! { Text("nested") };,
            },
            true,
        );
        let expected = quote! {
            ::rustui::layout::VStack::new()
                .child(Text("first").size(14.0))
                .child(Button("second", self.action(Self::hit)).id("button"))
                .child(HStack! { Text("nested") })
        };
        assert_same_expression(actual, expected);
    }

    #[test]
    fn preserves_method_calls_on_existing_values_and_optional_children() {
        let actual = expanded(
            quote! {
                retention_controls.spacing(8.0)
                visible.then(|| Component("child", move || Child::new(probe.clone())))
                if show { existing.disabled(false) }
            },
            true,
        );
        let expected = quote! {
            ::rustui::layout::VStack::new()
                .child(retention_controls.spacing(8.0))
                .child(visible.then(|| Component("child", move || Child::new(probe.clone()))))
                .group(if show {
                    ::rustui::layout::Group::new().child(existing.disabled(false))
                } else { ::rustui::layout::Group::new() })
        };
        assert_same_expression(actual, expected);
    }

    #[test]
    fn adjacent_parenthesized_optional_child_is_not_a_call_on_previous_view() {
        let actual = expanded(
            quote! {
                controls.spacing(8.0)
                (selected == StoryId::Layout && step() == 0)
                    .then(|| Component("foundation", Foundation::default))
                Text("after")
                if show {
                    Text("inside")
                    (ready && !pending).then(|| Text("optional"))
                }
            },
            true,
        );
        let expected = quote! {
            ::rustui::layout::VStack::new()
                .child(controls.spacing(8.0))
                .child((selected == StoryId::Layout && step() == 0)
                    .then(|| Component("foundation", Foundation::default)))
                .child(Text("after"))
                .group(if show {
                    ::rustui::layout::Group::new()
                        .child(Text("inside"))
                        .child((ready && !pending).then(|| Text("optional")))
                } else { ::rustui::layout::Group::new() })
        };
        assert_same_expression(actual, expected);
    }

    #[test]
    fn if_without_else_flattens_to_a_group_or_no_children() {
        let actual = expanded(
            quote! {
                Text("before")
                if visible { Text("one") TextField("two", binding()).id("input") }
                Text("after")
            },
            true,
        );
        let expected = quote! {
            ::rustui::layout::VStack::new()
                .child(Text("before"))
                .group(if visible {
                    ::rustui::layout::Group::new()
                        .child(Text("one"))
                        .child(TextField("two", binding()).id("input"))
                } else { ::rustui::layout::Group::new() })
                .child(Text("after"))
        };
        assert_same_expression(actual, expected);
    }

    #[test]
    fn heterogeneous_else_if_branches_preserve_short_circuit_structure() {
        let actual = expanded(
            quote! {
                if primary() { Text("primary") }
                else if secondary() { Button("secondary", hit) }
                else { TextField("fallback", binding()) }
            },
            false,
        );
        let expected = quote! {
            ::rustui::layout::HStack::new().group(if primary() {
                ::rustui::layout::Group::new().child(Text("primary"))
            } else {
                if secondary() {
                    ::rustui::layout::Group::new()
                        .child(Button("secondary", self.action(Self::hit)))
                } else {
                    ::rustui::layout::Group::new().child(TextField("fallback", binding()))
                }
            })
        };
        assert_same_expression(actual, expected);
    }

    #[test]
    fn nested_branches_keep_explicit_keys_and_rewrite_nested_actions() {
        let actual = expanded(
            quote! {
                if outer {
                    if inner {
                        Keyed("editor", TextEditor("Edit", binding()).id("shared"))
                        Keyed("action", Button("Hit", hit))
                    } else {}
                    VStack! { if nested { Button("Nested", hit) } }
                }
            },
            true,
        )
        .to_string();
        assert!(actual.contains(
            &quote!(Keyed("editor", TextEditor("Edit", binding()).id("shared"))).to_string()
        ));
        assert!(
            actual.contains(
                &quote!(Keyed("action", Button("Hit", self.action(Self::hit)))).to_string()
            )
        );
        // The nested proc macro expands itself; its input must remain intact.
        assert!(
            actual.contains(&quote!(VStack! { if nested { Button("Nested", hit) } }).to_string())
        );
        assert!(!actual.contains("RetentionScope"));
    }

    #[test]
    fn explicit_and_qualified_actions_are_not_rewritten() {
        let actual = expanded(
            quote! {
                if show {
                    Button("Explicit", self.action(Self::hit))
                    Button("Qualified handler", actions::hit)
                    rustui::Button("Qualified constructor", hit)
                }
            },
            true,
        )
        .to_string();
        assert_eq!(actual.matches("self . action").count(), 1);
        assert!(actual.contains(&quote!(Button("Qualified handler", actions::hit)).to_string()));
        assert!(actual.contains(&quote!(rustui::Button("Qualified constructor", hit)).to_string()));
    }

    #[test]
    fn condition_is_emitted_once_and_empty_branches_are_valid() {
        let actual = expanded(quote! { if next_condition() {} else {} }, true).to_string();
        assert_eq!(actual.matches("next_condition").count(), 1);
        assert_eq!(actual.matches("Group :: new").count(), 2);
        let empty = expanded(quote! { ,; }, false);
        assert_eq!(
            empty.to_string(),
            quote!(::rustui::layout::HStack::new()).to_string()
        );
    }

    #[test]
    fn rejects_malformed_conditionals_instead_of_dropping_tokens() {
        for input in [
            quote! { if visible Text("missing braces") },
            quote! { if visible {} else Text("missing braces") },
            quote! { if visible {} else if {} },
            quote! { if visible { Text("ok") } else {} else {} },
            quote! { if visible { let value = Text("statement"); } },
        ] {
            assert!(expand_with(input, true).is_err());
        }
    }

    #[test]
    fn rejects_unimplemented_for_match_and_bare_value_grammar() {
        for input in [
            quote! { for item in items { Text(item) } },
            quote! { match value { _ => Text("value") } },
            quote! { if visible { for item in items { Text(item) } } },
            quote! { child },
            quote! { { Text("block") } },
            quote! { (Text("tuple"), Text("tuple")) },
        ] {
            assert!(expand_with(input, false).is_err());
        }
    }
}
