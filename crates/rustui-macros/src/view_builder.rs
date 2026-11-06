use proc_macro::TokenStream;
use proc_macro2::{Delimiter, Ident, Span, TokenStream as TokenStream2, TokenTree};
use quote::quote;
use syn::{
    Attribute, Expr, Result, Token, braced,
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
        if handler.qself.is_some()
            || handler.path.segments.len() != 1
            || !matches!(
                &handler.path.segments[0].arguments,
                syn::PathArguments::None
            )
        {
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
                if input.peek(Token![for])
                    || input.peek(Token![match])
                    || input.peek(Token![while])
                    || input.peek(Token![loop])
                {
                    return Err(input.error(
                        "stack macros do not support `for`, `match`, `while`, or `loop`; expand those branches before the stack macro",
                    ));
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
    let first_group = input.fork().parse::<TokenTree>().ok();
    let mut parenthesized_callee = matches!(first_group,
        Some(TokenTree::Group(ref group)) if group.delimiter() == Delimiter::Parenthesis
            && syn::parse2::<Expr>(group.stream()).is_ok_and(|expression| {
                !is_constructor(&expression) || is_macro_expression(&expression)
            })
    );
    let mut macro_bang = false;
    let mut generic_depth = 0usize;
    let mut previous_was_type_arrow = false;
    while !input.is_empty() {
        let token: TokenTree = input.parse()?;
        let closes_constructor = matches!(
            &token,
            TokenTree::Group(group)
                if generic_depth == 0
                    && (matches!(group.delimiter(), Delimiter::Parenthesis | Delimiter::Brace)
                        || (macro_bang && group.delimiter() == Delimiter::Bracket))
        );
        let is_type_arrow = previous_was_type_arrow
            && matches!(&token, TokenTree::Punct(punct) if punct.as_char() == '>');
        if let TokenTree::Punct(punct) = &token {
            match punct.as_char() {
                '<' => generic_depth += 1,
                '>' if generic_depth > 0 && !is_type_arrow => generic_depth -= 1,
                _ => {}
            }
        }
        previous_was_type_arrow = matches!(&token, TokenTree::Punct(punct) if punct.as_char() == '-' && punct.spacing() == proc_macro2::Spacing::Joint);
        macro_bang = matches!(&token, TokenTree::Punct(punct) if punct.as_char() == '!');
        tokens.extend([token]);
        if closes_constructor {
            break;
        }
    }
    loop {
        let call_arguments_follow = matches!(
            input.fork().parse::<TokenTree>(),
            Ok(TokenTree::Group(group)) if group.delimiter() == Delimiter::Parenthesis
        );
        let current_is_constructor =
            syn::parse2::<Expr>(tokens.clone()).is_ok_and(|expression| is_constructor(&expression));
        if call_arguments_follow
            && (parenthesized_callee || current_is_constructor)
            && !parenthesized_method_child_follows(input)
            && (parenthesized_callee || !parenthesized_constructor_child_follows(input))
        {
            tokens.extend([input.parse::<TokenTree>()?]);
            parenthesized_callee = false;
            continue;
        }
        if matches!(
            input.fork().parse::<TokenTree>(),
            Ok(TokenTree::Group(group)) if group.delimiter() == Delimiter::Bracket
        ) {
            tokens.extend([input.parse::<TokenTree>()?]);
            continue;
        }
        if input.peek(Token![?]) {
            tokens.extend([input.parse::<TokenTree>()?]);
            continue;
        }
        let await_suffix = input.fork();
        if await_suffix.peek(Token![.]) {
            await_suffix.parse::<Token![.]>().ok();
            if await_suffix.peek(Token![await]) {
                tokens.extend([input.parse::<TokenTree>()?]);
                tokens.extend([input.parse::<TokenTree>()?]);
                continue;
            }
        }
        if let Some(token_count) = field_access_token_count(input) {
            for _ in 0..token_count {
                tokens.extend([input.parse::<TokenTree>()?]);
            }
            continue;
        }
        let Some(token_count) = method_call_token_count(input) else {
            break;
        };
        for _ in 0..token_count {
            tokens.extend([input.parse::<TokenTree>()?]);
        }
    }
    syn::parse2(tokens)
}

fn is_macro_expression(expression: &Expr) -> bool {
    match expression {
        Expr::Macro(_) => true,
        Expr::Paren(expression) => is_macro_expression(&expression.expr),
        _ => false,
    }
}

fn parenthesized_method_child_follows(input: ParseStream<'_>) -> bool {
    let lookahead = input.fork();
    if !matches!(
        lookahead.parse::<TokenTree>(),
        Ok(TokenTree::Group(group)) if group.delimiter() == Delimiter::Parenthesis
    ) || !lookahead.peek(Token![.])
    {
        return false;
    }
    lookahead.parse::<Token![.]>().ok();
    if !matches!(lookahead.parse::<Ident>(), Ok(method) if method != "await") {
        return false;
    }
    if lookahead.peek(Token![::])
        && syn::AngleBracketedGenericArguments::parse_turbofish(&lookahead).is_err()
    {
        return false;
    }
    matches!(
        lookahead.parse::<TokenTree>(),
        Ok(TokenTree::Group(group)) if group.delimiter() == Delimiter::Parenthesis
    )
}

fn parenthesized_constructor_child_follows(input: ParseStream<'_>) -> bool {
    let lookahead = input.fork();
    let Ok(TokenTree::Group(group)) = lookahead.parse::<TokenTree>() else {
        return false;
    };
    group.delimiter() == Delimiter::Parenthesis
        && syn::parse2::<Expr>(group.stream()).is_ok_and(|expression| is_constructor(&expression))
}

fn field_access_token_count(input: ParseStream<'_>) -> Option<usize> {
    let lookahead = input.fork();
    if !matches!(lookahead.parse::<TokenTree>(), Ok(TokenTree::Punct(dot)) if dot.as_char() == '.')
    {
        return None;
    }

    match lookahead.parse::<TokenTree>().ok()? {
        TokenTree::Ident(field) if field != "await" => {
            if lookahead.peek(Token![::]) || lookahead.peek(syn::token::Paren) {
                return None;
            }
        }
        _ => {
            let index = input.fork();
            index.parse::<Token![.]>().ok()?;
            syn::Index::parse(&index).ok()?;
            let end = index.cursor();
            let consumed = input.fork();
            let mut count = 0;
            while consumed.cursor() != end {
                consumed.parse::<TokenTree>().ok()?;
                count += 1;
            }
            return Some(count);
        }
    }

    let end = lookahead.cursor();
    let consumed = input.fork();
    let mut count = 0;
    while consumed.cursor() != end {
        consumed.parse::<TokenTree>().ok()?;
        count += 1;
    }
    Some(count)
}

fn method_call_token_count(input: ParseStream<'_>) -> Option<usize> {
    let lookahead = input.fork();
    if !matches!(lookahead.parse::<TokenTree>(), Ok(TokenTree::Punct(dot)) if dot.as_char() == '.')
        || !matches!(lookahead.parse::<TokenTree>(), Ok(TokenTree::Ident(_)))
    {
        return None;
    }

    if lookahead.peek(Token![::]) {
        syn::AngleBracketedGenericArguments::parse_turbofish(&lookahead).ok()?;
    }

    if !matches!(
        lookahead.parse::<TokenTree>(),
        Ok(TokenTree::Group(arguments)) if arguments.delimiter() == Delimiter::Parenthesis
    ) {
        return None;
    }

    let end = lookahead.cursor();
    let consumed = input.fork();
    let mut count = 0;
    while consumed.cursor() != end {
        consumed.parse::<TokenTree>().ok()?;
        count += 1;
    }
    Some(count)
}

fn is_constructor(expression: &Expr) -> bool {
    match expression {
        Expr::Call(_) | Expr::MethodCall(_) | Expr::Macro(_) | Expr::Struct(_) => true,
        Expr::Try(expression) => is_constructor(&expression.expr),
        Expr::Await(expression) => is_constructor(&expression.base),
        Expr::Index(expression) => is_constructor(&expression.expr),
        Expr::Field(expression) => is_constructor(&expression.base),
        Expr::Paren(expression) => is_constructor(&expression.expr),
        _ => false,
    }
}

fn constructor_attributes(expression: &Expr) -> &[Attribute] {
    match expression {
        Expr::Call(expression) => &expression.attrs,
        Expr::MethodCall(expression) => &expression.attrs,
        Expr::Macro(expression) => &expression.attrs,
        Expr::Struct(expression) => &expression.attrs,
        Expr::Try(expression) => &expression.attrs,
        Expr::Await(expression) => &expression.attrs,
        Expr::Index(expression) => &expression.attrs,
        Expr::Field(expression) => &expression.attrs,
        Expr::Paren(expression) => &expression.attrs,
        _ => &[],
    }
}

fn constructor_attributes_mut(expression: &mut Expr) -> &mut Vec<Attribute> {
    match expression {
        Expr::Call(expression) => &mut expression.attrs,
        Expr::MethodCall(expression) => &mut expression.attrs,
        Expr::Macro(expression) => &mut expression.attrs,
        Expr::Struct(expression) => &mut expression.attrs,
        Expr::Try(expression) => &mut expression.attrs,
        Expr::Await(expression) => &mut expression.attrs,
        Expr::Index(expression) => &mut expression.attrs,
        Expr::Field(expression) => &mut expression.attrs,
        Expr::Paren(expression) => &mut expression.attrs,
        _ => unreachable!("only stack constructors have child attributes"),
    }
}

fn gates_builder(attribute: &Attribute) -> bool {
    fn contains_cfg(meta: &syn::Meta) -> bool {
        match meta {
            syn::Meta::List(list) if list.path.is_ident("cfg") => true,
            syn::Meta::List(list) if list.path.is_ident("cfg_attr") => {
                let Ok(arguments) = list.parse_args_with(
                    syn::punctuated::Punctuated::<syn::Meta, Token![,]>::parse_terminated,
                ) else {
                    return true;
                };
                let mut arguments = arguments.into_iter();
                if arguments.next().is_none() {
                    return true;
                }
                arguments.any(|attribute| contains_cfg(&attribute))
            }
            _ => false,
        }
    }

    contains_cfg(&attribute.meta)
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
                StackChild::View(view) => {
                    let conditional_attributes = constructor_attributes(view)
                        .iter()
                        .filter(|attribute| gates_builder(attribute))
                        .cloned()
                        .collect::<Vec<_>>();

                    if conditional_attributes.is_empty() {
                        quote!(#builder.child(#view))
                    } else {
                        let builder_ident =
                            Ident::new("__rustui_stack_builder", Span::mixed_site());
                        let mut view = view.clone();
                        constructor_attributes_mut(&mut view)
                            .retain(|attribute| !gates_builder(attribute));
                        quote!({
                            let #builder_ident = #builder;
                            #(#conditional_attributes)*
                            let #builder_ident = #builder_ident.child(#view);
                            #builder_ident
                        })
                    }
                }
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
    fn bracket_delimited_macro_child_stops_before_the_next_sibling() {
        let actual = expanded(quote! { make_view!["macro child"] Text("sibling") }, true);
        let expected = quote! {
            ::rustui::layout::VStack::new()
                .child(make_view!["macro child"])
                .child(Text("sibling"))
        };
        assert_same_expression(actual, expected);
    }

    #[test]
    fn parenthesized_constructor_remains_a_child_and_keeps_its_cfg_boundary() {
        let actual = expanded(
            quote! {
                #[cfg(any())]
                (Text("wrapped"))
                Text("sibling")
            },
            true,
        );
        let builder = Ident::new("__rustui_stack_builder", Span::mixed_site());
        let expected = quote! {
            {
                let #builder = ::rustui::layout::VStack::new();
                #[cfg(any())]
                let #builder = #builder.child((Text("wrapped")));
                #builder
            }
            .child(Text("sibling"))
        };
        assert_same_expression(actual, expected);
    }

    #[test]
    fn parenthesized_callable_constructor_keeps_its_call_arguments() {
        let actual = expanded(quote! { (make_view)("first") Text("sibling") }, true);
        let expected = quote! {
            ::rustui::layout::VStack::new()
                .child((make_view)("first"))
                .child(Text("sibling"))
        };
        assert_same_expression(actual, expected);
    }

    #[test]
    fn parenthesized_macro_callable_keeps_its_call_arguments() {
        let actual = expanded(quote! { (make_view![])("first") Text("sibling") }, true);
        let expected = quote! {
            ::rustui::layout::VStack::new()
                .child((make_view![])("first"))
                .child(Text("sibling"))
        };
        assert_same_expression(actual, expected);
    }

    #[test]
    fn chained_call_keeps_its_arguments_and_the_next_constructor_sibling() {
        let actual = expanded(quote! { make_view()("label") Text("sibling") }, true);
        let expected = quote! {
            ::rustui::layout::VStack::new()
                .child(make_view()("label"))
                .child(Text("sibling"))
        };
        assert_same_expression(actual, expected);

        let actual = expanded(quote! { make_view() (Text("sibling")) }, true);
        let expected = quote! {
            ::rustui::layout::VStack::new()
                .child(make_view())
                .child((Text("sibling")))
        };
        assert_same_expression(actual, expected);
    }

    #[test]
    fn try_wrapped_call_keeps_its_arguments_and_the_next_constructor_sibling() {
        let actual = expanded(quote! { make_view()? ("label") Text("sibling") }, true);
        let expected = quote! {
            ::rustui::layout::VStack::new()
                .child(make_view()? ("label"))
                .child(Text("sibling"))
        };
        assert_same_expression(actual, expected);
    }

    #[test]
    fn parenthesized_macro_child_keeps_the_adjacent_optional_child_boundary() {
        let actual = expanded(
            quote! {
                (make_view![])
                (visible).then(|| Text("optional"))
            },
            true,
        );
        let expected = quote! {
            ::rustui::layout::VStack::new()
                .child((make_view![]))
                .child((visible).then(|| Text("optional")))
        };
        assert_same_expression(actual, expected);
    }

    #[test]
    fn then_some_optional_child_keeps_adjacent_constructor_siblings() {
        let actual = expanded(
            quote! {
                Text("before")
                (visible).then_some(Text("optional"))
                Text("after")
            },
            true,
        );
        let expected = quote! {
            ::rustui::layout::VStack::new()
                .child(Text("before"))
                .child((visible).then_some(Text("optional")))
                .child(Text("after"))
        };
        assert_same_expression(actual, expected);
    }

    #[test]
    fn turbofish_then_some_optional_child_keeps_adjacent_constructor_siblings() {
        let actual = expanded(
            quote! {
                Text("before")
                (visible).then_some::<TextView>(Text("optional"))
                Text("after")
            },
            true,
        );
        let expected = quote! {
            ::rustui::layout::VStack::new()
                .child(Text("before"))
                .child((visible).then_some::<TextView>(Text("optional")))
                .child(Text("after"))
        };
        assert_same_expression(actual, expected);
    }

    #[test]
    fn parenthesized_and_then_optional_child_keeps_adjacent_constructor_siblings() {
        let actual = expanded(
            quote! {
                Text("before")
                (optional).and_then(|child| Some(child))
                Text("after")
            },
            true,
        );
        let expected = quote! {
            ::rustui::layout::VStack::new()
                .child(Text("before"))
                .child((optional).and_then(|child| Some(child)))
                .child(Text("after"))
        };
        assert_same_expression(actual, expected);
    }

    #[test]
    fn generic_constructor_function_pointer_type_does_not_end_the_child() {
        let actual = expanded(quote! { make_view::<fn() -> u8>() Text("sibling") }, true);
        let expected = quote! {
            ::rustui::layout::VStack::new()
                .child(make_view::<fn() -> u8>())
                .child(Text("sibling"))
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
    fn try_operator_stays_with_its_constructor_before_the_next_sibling() {
        let actual = expanded(
            quote! {
                #[cfg(any())]
                make_view()?
                Text("sibling")
            },
            true,
        );
        let builder = Ident::new("__rustui_stack_builder", Span::mixed_site());
        let expected = quote! {
            {
                let #builder = ::rustui::layout::VStack::new();
                #[cfg(any())]
                let #builder = #builder.child(make_view()?);
                #builder
            }
            .child(Text("sibling"))
        };
        assert_same_expression(actual, expected);
    }

    #[test]
    fn await_suffix_stays_with_its_constructor_before_the_next_sibling() {
        let actual = expanded(
            quote! {
                #[cfg(any())]
                make_view().await
                Text("sibling")
            },
            true,
        );
        let builder = Ident::new("__rustui_stack_builder", Span::mixed_site());
        let expected = quote! {
            {
                let #builder = ::rustui::layout::VStack::new();
                #[cfg(any())]
                let #builder = #builder.child(make_view().await);
                #builder
            }
            .child(Text("sibling"))
        };
        assert_same_expression(actual, expected);
    }

    #[test]
    fn index_suffix_stays_with_its_constructor_before_the_next_sibling() {
        let actual = expanded(
            quote! {
                make_views()[0]
                Text("sibling")
            },
            true,
        );
        let expected = quote! {
            ::rustui::layout::VStack::new()
                .child(make_views()[0])
                .child(Text("sibling"))
        };
        assert_same_expression(actual, expected);
    }

    #[test]
    fn field_suffix_stays_with_its_constructor_before_the_next_sibling() {
        let actual = expanded(
            quote! {
                #[cfg(any())]
                make_views().0.view
                Text("sibling")
            },
            true,
        );
        let builder = Ident::new("__rustui_stack_builder", Span::mixed_site());
        let expected = quote! {
            {
                let #builder = ::rustui::layout::VStack::new();
                #[cfg(any())]
                let #builder = #builder.child(make_views().0.view);
                #builder
            }
            .child(Text("sibling"))
        };
        assert_same_expression(actual, expected);
    }

    #[test]
    fn preserves_block_bodied_optional_children_with_nested_closures() {
        let actual = expanded(
            quote! {
                visible.then(|| {
                    Component("child", move || {
                        PendingTaskChild::new(probe.clone())
                    })
                })
            },
            true,
        );
        let expected = quote! {
            ::rustui::layout::VStack::new()
                .child(visible.then(|| {
                    Component("child", move || {
                        PendingTaskChild::new(probe.clone())
                    })
                }))
        };
        assert_same_expression(actual, expected);
    }

    #[test]
    fn preserves_turbofish_method_chains_on_nested_views() {
        let actual = expanded(
            quote! { VStack::new().child::<TextView>(Text("nested")) },
            true,
        );
        let expected = quote! {
            ::rustui::layout::VStack::new()
                .child(VStack::new().child::<TextView>(Text("nested")))
        };
        assert_same_expression(actual, expected);
    }

    #[test]
    fn preserves_turbofish_function_pointer_return_types() {
        let actual = expanded(quote! { Text("generic").configure::<fn() -> u8>() }, true);
        let expected = quote! {
            ::rustui::layout::VStack::new()
                .child(Text("generic").configure::<fn() -> u8>())
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
    fn conditional_child_attributes_gate_the_child_builder_call() {
        let actual = expanded(
            quote! {
                #[cfg(any())]
                Text("conditional")
                #[cfg_attr(any(), cfg(any()))]
                Button("conditional", hit)
            },
            true,
        );
        let builder = Ident::new("__rustui_stack_builder", Span::mixed_site());
        let expected = quote! {
            {
                let #builder = {
                    let #builder = ::rustui::layout::VStack::new();
                    #[cfg(any())]
                    let #builder = #builder.child(Text("conditional"));
                    #builder
                };
                #[cfg_attr(any(), cfg(any()))]
                let #builder = #builder.child(Button("conditional", self.action(Self::hit)));
                #builder
            }
        };
        assert_same_expression(actual, expected);
    }

    #[test]
    fn docs_only_cfg_attr_stays_on_the_child_expression() {
        let actual = expanded(
            quote! {
                #[cfg_attr(doc, doc = "child documentation")]
                Text("documented")
            },
            true,
        );
        let expected = quote! {
            ::rustui::layout::VStack::new()
                .child(#[cfg_attr(doc, doc = "child documentation")] Text("documented"))
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
                Button("Generic handler", save::<Commit>)
                    rustui::Button("Qualified constructor", hit)
                }
            },
            true,
        )
        .to_string();
        assert_eq!(actual.matches("self . action").count(), 1);
        assert!(actual.contains(&quote!(Button("Qualified handler", actions::hit)).to_string()));
        assert!(actual.contains(&quote!(Button("Generic handler", save::<Commit>)).to_string()));
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

    #[test]
    fn unsupported_control_flow_diagnostic_explains_the_supported_boundary() {
        for input in [
            quote! { for item in items { Text(item) } },
            quote! { while active { Text("value") } },
            quote! { loop { Text("value") } },
            quote! { if visible { match value { _ => Text("value") } } },
        ] {
            let error = expand_with(input, true).unwrap_err();
            assert_eq!(
                error.to_string(),
                "stack macros do not support `for`, `match`, `while`, or `loop`; expand those branches before the stack macro"
            );
        }
    }
}
