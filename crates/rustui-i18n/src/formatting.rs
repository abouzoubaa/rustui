//! Named interpolation, plural selection, text selection, and integer formatting.
//!
//! Placeholders use `{name}` syntax. Double braces escape literal braces
//! outside expression bodies. Inside expression bodies, doubled opening braces
//! and `\}` escape literal closing braces. Every
//! placeholder must have a supplied argument; extra arguments are ignored.
//! Expressions use `{name, plural, key {message} other {message}}`,
//! `{name, selectordinal, key {message} other {message}}`, or
//! `{name, select, key {message} other {message}}`. `#` inside a plural or ordinal branch
//! inserts the active plural value; `\#` escapes a literal pound sign there.
//! Expression bodies can contain ordinary placeholders and nested expressions. Cardinal categories
//! are CLDR integer rules for English, French, and Arabic; other languages use
//! `other` unless an exact `=integer` branch matches.

use crate::{Locale, direction::explicit_script};
use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fmt,
};

/// A value that can be inserted into a catalog message.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MessageArgument {
    Text(String),
    Integer(i64),
    Decimal(String),
}

impl From<String> for MessageArgument {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

impl From<&str> for MessageArgument {
    fn from(value: &str) -> Self {
        Self::Text(value.to_owned())
    }
}

impl From<i64> for MessageArgument {
    fn from(value: i64) -> Self {
        Self::Integer(value)
    }
}

impl From<i32> for MessageArgument {
    fn from(value: i32) -> Self {
        Self::Integer(i64::from(value))
    }
}

/// Named arguments supplied to [`interpolate`].
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MessageArguments(BTreeMap<String, MessageArgument>);

impl From<f64> for MessageArgument {
    fn from(value: f64) -> Self {
        Self::Decimal(value.to_string())
    }
}

impl From<f32> for MessageArgument {
    fn from(value: f32) -> Self {
        Self::Decimal(value.to_string())
    }
}

impl MessageArguments {
    pub(crate) fn iter(&self) -> impl Iterator<Item = (&str, &MessageArgument)> {
        self.0.iter().map(|(name, value)| (name.as_str(), value))
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, name: impl Into<String>, value: impl Into<MessageArgument>) {
        self.0.insert(name.into(), value.into());
    }

    pub fn with(mut self, name: impl Into<String>, value: impl Into<MessageArgument>) -> Self {
        self.insert(name, value);
        self
    }
}

/// Failure to parse a message or find/type-check its required argument.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InterpolationError {
    Fluent(crate::FluentFormatError),
    InvalidPlaceholder(String),
    MissingArgument(String),
    InvalidMessage(String),
    WrongArgumentType(String),
    UnmatchedOpeningBrace,
    UnmatchedClosingBrace,
}

impl fmt::Display for InterpolationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Fluent(error) => error.fmt(formatter),
            Self::InvalidPlaceholder(name) => write!(formatter, "invalid placeholder `{{{name}}}`"),
            Self::MissingArgument(name) => write!(formatter, "missing message argument `{name}`"),
            Self::InvalidMessage(reason) => {
                write!(formatter, "invalid message expression: {reason}")
            }
            Self::WrongArgumentType(name) => {
                write!(formatter, "message argument `{name}` has the wrong type")
            }
            Self::UnmatchedOpeningBrace => {
                formatter.write_str("unmatched opening brace in message")
            }
            Self::UnmatchedClosingBrace => {
                formatter.write_str("unmatched closing brace in message")
            }
        }
    }
}

impl Error for InterpolationError {}

/// Inserts named arguments and evaluates plural, selectordinal, and select expressions using the default English rules.
///
/// Numeric arguments use ordinary decimal formatting, independent of locale.
/// Literal braces are written as `{{` and `}}`.
pub fn interpolate(
    template: &str,
    arguments: &MessageArguments,
) -> Result<String, InterpolationError> {
    interpolate_inner(template, arguments, None)
}

/// Interpolates a message, formatting numeric arguments for the supplied locale.
/// English, French, and Arabic receive grouped integers; other locales retain
/// invariant ASCII decimal formatting.
pub fn interpolate_with_locale(
    template: &str,
    arguments: &MessageArguments,
    locale: &Locale,
) -> Result<String, InterpolationError> {
    interpolate_inner(template, arguments, Some(locale))
}

fn interpolate_inner(
    template: &str,
    arguments: &MessageArguments,
    locale: Option<&Locale>,
) -> Result<String, InterpolationError> {
    let parsed = parse_message(template)?;
    let format_locale = locale;
    let default_locale;
    let rule_locale = match locale {
        Some(locale) => locale,
        None => {
            default_locale = Locale::parse("en").expect("English locale is valid");
            &default_locale
        }
    };
    render_nodes(&parsed.nodes, arguments, rule_locale, format_locale, None)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum ExpressionKind {
    Plural,
    Ordinal,
    Select,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Node {
    Text(String),
    Argument(String),
    Pound,
    Expression {
        name: String,
        kind: ExpressionKind,
        branches: BTreeMap<String, Vec<Node>>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ParsedMessage {
    nodes: Vec<Node>,
    contract: MessageContract,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct ExpressionSiteKey {
    name: String,
    kind: ExpressionKind,
    ordinal: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
struct ContractSequence {
    arguments: BTreeSet<String>,
    expressions: BTreeMap<ExpressionSiteKey, ExpressionContract>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct ExpressionContract {
    branches: BTreeMap<String, ContractSequence>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct CanonicalSequence {
    arguments: BTreeSet<String>,
    expressions: BTreeMap<ExpressionSiteKey, CanonicalExpression>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct CanonicalExpression {
    branches: CanonicalBranches,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum CanonicalBranches {
    Select(BTreeMap<String, CanonicalSequence>),
    Plural {
        exact: BTreeMap<String, CanonicalSequence>,
        categories: BTreeSet<CanonicalSequence>,
    },
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct MessageContract {
    arguments: BTreeSet<String>,
    expressions: ContractSequence,
}

impl MessageContract {
    pub(crate) fn argument_names(&self) -> BTreeSet<String> {
        self.arguments.clone()
    }
    pub(crate) fn arguments_match(&self, other: &Self) -> bool {
        self.arguments == other.arguments
    }

    pub(crate) fn matches(&self, other: &Self) -> bool {
        matches_sequence(&self.expressions, &other.expressions)
    }
}

fn matches_sequence(left: &ContractSequence, right: &ContractSequence) -> bool {
    if left.arguments != right.arguments || left.expressions.keys().ne(right.expressions.keys()) {
        return false;
    }

    left.expressions.iter().all(|(site, left_expression)| {
        right.expressions.get(site).is_some_and(|right_expression| {
            matches_expression(site.kind, left_expression, right_expression)
        })
    })
}

fn matches_expression(
    kind: ExpressionKind,
    left: &ExpressionContract,
    right: &ExpressionContract,
) -> bool {
    match kind {
        ExpressionKind::Select => {
            left.branches.keys().eq(right.branches.keys())
                && left.branches.iter().all(|(key, left_body)| {
                    right
                        .branches
                        .get(key)
                        .is_some_and(|right_body| matches_sequence(left_body, right_body))
                })
        }
        ExpressionKind::Plural | ExpressionKind::Ordinal => {
            let left_exact = plural_exact_branches(left);
            let right_exact = plural_exact_branches(right);
            if left_exact.keys().ne(right_exact.keys()) {
                return false;
            }
            if !left_exact.iter().all(|(key, left_body)| {
                right_exact
                    .get(key)
                    .is_some_and(|right_body| matches_sequence(left_body, right_body))
            }) {
                return false;
            }

            if !left.branches.iter().all(|(key, left_body)| {
                key.starts_with('=')
                    || right
                        .branches
                        .get(key)
                        .is_none_or(|right_body| matches_sequence(left_body, right_body))
            }) {
                return false;
            }
            if !right.branches.iter().all(|(key, right_body)| {
                key.starts_with('=')
                    || left
                        .branches
                        .get(key)
                        .is_none_or(|left_body| matches_sequence(left_body, right_body))
            }) {
                return false;
            }

            canonical_plural_categories(left) == canonical_plural_categories(right)
        }
    }
}

fn plural_exact_branches(expression: &ExpressionContract) -> BTreeMap<&str, &ContractSequence> {
    expression
        .branches
        .iter()
        .filter(|(key, _)| key.starts_with('='))
        .map(|(key, body)| (key.as_str(), body))
        .collect()
}

fn canonical_plural_categories(expression: &ExpressionContract) -> BTreeSet<CanonicalSequence> {
    expression
        .branches
        .iter()
        .filter(|(key, _)| !key.starts_with('='))
        .map(|(_, body)| canonicalize_sequence(body))
        .collect()
}

fn canonicalize_sequence(sequence: &ContractSequence) -> CanonicalSequence {
    CanonicalSequence {
        arguments: sequence.arguments.clone(),
        expressions: sequence
            .expressions
            .iter()
            .map(|(site, expression)| {
                (site.clone(), canonicalize_expression(site.kind, expression))
            })
            .collect(),
    }
}

fn canonicalize_expression(
    kind: ExpressionKind,
    expression: &ExpressionContract,
) -> CanonicalExpression {
    let branches = match kind {
        ExpressionKind::Select => CanonicalBranches::Select(
            expression
                .branches
                .iter()
                .map(|(key, body)| (key.clone(), canonicalize_sequence(body)))
                .collect(),
        ),
        ExpressionKind::Plural | ExpressionKind::Ordinal => CanonicalBranches::Plural {
            exact: plural_exact_branches(expression)
                .into_iter()
                .map(|(key, body)| (key.to_owned(), canonicalize_sequence(body)))
                .collect(),
            categories: canonical_plural_categories(expression),
        },
    };
    CanonicalExpression { branches }
}

pub(crate) fn message_contract(template: &str) -> Result<MessageContract, InterpolationError> {
    Ok(parse_message(template)?.contract)
}

fn parse_message(template: &str) -> Result<ParsedMessage, InterpolationError> {
    let mut parser = Parser {
        chars: template.chars().collect(),
        index: 0,
    };
    let nodes = parser.nodes(false, false)?;
    if parser.index != parser.chars.len() {
        return Err(InterpolationError::UnmatchedClosingBrace);
    }
    let mut contract = MessageContract::default();
    collect_contract(&nodes, &mut contract.arguments, &mut contract.expressions);
    Ok(ParsedMessage { nodes, contract })
}

struct Parser {
    chars: Vec<char>,
    index: usize,
}

impl Parser {
    fn nodes(
        &mut self,
        branch: bool,
        plural_context: bool,
    ) -> Result<Vec<Node>, InterpolationError> {
        let mut nodes = Vec::new();
        let mut text = String::new();
        while self.index < self.chars.len() {
            match self.chars[self.index] {
                '}' if branch => {
                    self.index += 1;
                    break;
                }
                '\\' if branch
                    && plural_context
                    && self.chars.get(self.index + 1) == Some(&'#') =>
                {
                    self.index += 2;
                    text.push('#');
                }
                '\\' if branch && self.chars.get(self.index + 1) == Some(&'}') => {
                    self.index += 2;
                    text.push('}');
                }
                '}' if self.chars.get(self.index + 1) == Some(&'}') => {
                    self.index += 2;
                    text.push('}');
                }
                '}' => return Err(InterpolationError::UnmatchedClosingBrace),
                '{' if self.chars.get(self.index + 1) == Some(&'{') => {
                    self.index += 2;
                    text.push('{');
                }
                '{' => {
                    if !text.is_empty() {
                        nodes.push(Node::Text(std::mem::take(&mut text)));
                    }
                    nodes.push(self.expression(plural_context)?);
                }
                '#' if branch && plural_context => {
                    self.index += 1;
                    if !text.is_empty() {
                        nodes.push(Node::Text(std::mem::take(&mut text)));
                    }
                    nodes.push(Node::Pound);
                }
                ch => {
                    self.index += 1;
                    text.push(ch);
                }
            }
        }
        if branch && (self.index == self.chars.len() && self.chars.last() != Some(&'}')) {
            return Err(InterpolationError::UnmatchedOpeningBrace);
        }
        if !text.is_empty() {
            nodes.push(Node::Text(text));
        }
        Ok(nodes)
    }

    fn expression(&mut self, plural_context: bool) -> Result<Node, InterpolationError> {
        self.index += 1; // opening brace
        let name = self.token_until(&[',', '}'])?;
        let name = name.trim().to_owned();
        if !valid_argument_name(&name) {
            return Err(InterpolationError::InvalidPlaceholder(name));
        }
        match self.chars.get(self.index).copied() {
            Some('}') => {
                self.index += 1;
                Ok(Node::Argument(name))
            }
            Some(',') => {
                self.index += 1;
                let kind = match self.token_until(&[','])?.trim() {
                    "plural" => ExpressionKind::Plural,
                    "selectordinal" => ExpressionKind::Ordinal,
                    "select" => ExpressionKind::Select,
                    other => {
                        return Err(InterpolationError::InvalidMessage(format!(
                            "unknown expression `{other}`"
                        )));
                    }
                };
                self.index += 1; // comma
                let mut branches = BTreeMap::new();
                loop {
                    self.skip_space();
                    if self.chars.get(self.index) == Some(&'}') {
                        self.index += 1;
                        break;
                    }
                    let key = self.token_until(&['{'])?.trim().to_owned();
                    if !valid_branch_key(&key, kind) {
                        return Err(InterpolationError::InvalidMessage(format!(
                            "invalid branch key `{key}`"
                        )));
                    }
                    if self.chars.get(self.index) != Some(&'{') {
                        return Err(InterpolationError::UnmatchedOpeningBrace);
                    }
                    self.index += 1;
                    let body = self.nodes(
                        true,
                        plural_context
                            || matches!(kind, ExpressionKind::Plural | ExpressionKind::Ordinal),
                    )?;
                    if branches.insert(key.clone(), body).is_some() {
                        return Err(InterpolationError::InvalidMessage(format!(
                            "duplicate branch `{key}`"
                        )));
                    }
                }
                if !branches.contains_key("other") {
                    return Err(InterpolationError::InvalidMessage(
                        "expression requires an `other` branch".into(),
                    ));
                }
                Ok(Node::Expression {
                    name,
                    kind,
                    branches,
                })
            }
            _ => Err(InterpolationError::UnmatchedOpeningBrace),
        }
    }

    fn token_until(&mut self, delimiters: &[char]) -> Result<String, InterpolationError> {
        let start = self.index;
        while let Some(ch) = self.chars.get(self.index) {
            if delimiters.contains(ch) {
                break;
            }
            if *ch == '{' || *ch == '}' {
                return Err(InterpolationError::UnmatchedOpeningBrace);
            }
            self.index += 1;
        }
        if self.index == self.chars.len() {
            return Err(InterpolationError::UnmatchedOpeningBrace);
        }
        Ok(self.chars[start..self.index].iter().collect())
    }

    fn skip_space(&mut self) {
        while self
            .chars
            .get(self.index)
            .is_some_and(|ch| ch.is_whitespace())
        {
            self.index += 1;
        }
    }
}

fn valid_branch_key(key: &str, kind: ExpressionKind) -> bool {
    match kind {
        ExpressionKind::Plural => {
            matches!(key, "zero" | "one" | "two" | "few" | "many" | "other")
                || key.strip_prefix('=').is_some_and(|n| {
                    n.parse::<i64>()
                        .is_ok_and(|value| format!("={value}") == key)
                })
        }
        ExpressionKind::Ordinal => {
            matches!(key, "one" | "two" | "few" | "other")
                || key.strip_prefix('=').is_some_and(|n| {
                    n.parse::<i64>()
                        .is_ok_and(|value| format!("={value}") == key)
                })
        }
        ExpressionKind::Select => {
            !key.is_empty()
                && key
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
        }
    }
}

fn collect_contract(
    nodes: &[Node],
    arguments: &mut BTreeSet<String>,
    sequence: &mut ContractSequence,
) {
    let mut ordinals = BTreeMap::<(String, ExpressionKind), usize>::new();
    for node in nodes {
        match node {
            Node::Text(_) | Node::Pound => {}
            Node::Argument(name) => {
                arguments.insert(name.clone());
                sequence.arguments.insert(name.clone());
            }
            Node::Expression {
                name,
                kind,
                branches,
            } => {
                arguments.insert(name.clone());
                sequence.arguments.insert(name.clone());
                let ordinal = ordinals.entry((name.clone(), *kind)).or_default();
                let site = ExpressionSiteKey {
                    name: name.clone(),
                    kind: *kind,
                    ordinal: *ordinal,
                };
                *ordinal += 1;

                let branch_contracts = branches
                    .iter()
                    .map(|(key, body)| {
                        let mut child = ContractSequence::default();
                        collect_contract(body, arguments, &mut child);
                        (key.clone(), child)
                    })
                    .collect();
                sequence.expressions.insert(
                    site,
                    ExpressionContract {
                        branches: branch_contracts,
                    },
                );
            }
        }
    }
}

fn render_nodes(
    nodes: &[Node],
    args: &MessageArguments,
    rule_locale: &Locale,
    format_locale: Option<&Locale>,
    plural_argument: Option<&MessageArgument>,
) -> Result<String, InterpolationError> {
    let mut output = String::new();
    for node in nodes {
        match node {
            Node::Text(text) => output.push_str(text),
            Node::Argument(name) => {
                let value = args
                    .0
                    .get(name)
                    .ok_or_else(|| InterpolationError::MissingArgument(name.clone()))?;
                output.push_str(&format_argument(value, format_locale));
            }
            Node::Pound => output.push_str(&format_argument(
                plural_argument
                    .expect("pound nodes are parsed only inside plural or ordinal branches"),
                format_locale,
            )),
            Node::Expression {
                name,
                kind: ExpressionKind::Select,
                branches,
            } => {
                let Some(MessageArgument::Text(value)) = args.0.get(name) else {
                    if args.0.contains_key(name) {
                        return Err(InterpolationError::WrongArgumentType(name.clone()));
                    }
                    return Err(InterpolationError::MissingArgument(name.clone()));
                };
                let body = branches
                    .get(value)
                    .or_else(|| branches.get("other"))
                    .expect("parser requires other branch");
                output.push_str(&render_nodes(
                    body,
                    args,
                    rule_locale,
                    format_locale,
                    plural_argument,
                )?);
            }
            Node::Expression {
                name,
                kind: kind @ (ExpressionKind::Plural | ExpressionKind::Ordinal),
                branches,
            } => {
                let argument = args
                    .0
                    .get(name)
                    .ok_or_else(|| InterpolationError::MissingArgument(name.clone()))?;
                let key = match argument {
                    MessageArgument::Integer(number) => {
                        let exact = format!("={number}");
                        if branches.contains_key(&exact) {
                            exact
                        } else if *kind == ExpressionKind::Ordinal {
                            ordinal_category(*number, rule_locale).to_owned()
                        } else {
                            plural_category(*number, rule_locale).to_owned()
                        }
                    }
                    MessageArgument::Decimal(number) => {
                        let exact = exact_integer_from_decimal(number)
                            .map(|number| format!("={number}"))
                            .filter(|key| branches.contains_key(key));
                        exact.unwrap_or_else(|| {
                            if *kind == ExpressionKind::Ordinal {
                                ordinal_category_decimal(number, rule_locale).to_owned()
                            } else {
                                plural_category_decimal(number, rule_locale).to_owned()
                            }
                        })
                    }
                    MessageArgument::Text(_) => {
                        return Err(InterpolationError::WrongArgumentType(name.clone()));
                    }
                };
                let body = branches
                    .get(&key)
                    .or_else(|| branches.get("other"))
                    .expect("parser requires other branch");
                output.push_str(&render_nodes(
                    body,
                    args,
                    rule_locale,
                    format_locale,
                    Some(argument),
                )?);
            }
        }
    }
    Ok(output)
}

fn format_argument(argument: &MessageArgument, locale: Option<&Locale>) -> String {
    match argument {
        MessageArgument::Text(text) => text.clone(),
        MessageArgument::Integer(number) => match locale {
            Some(locale) => format_integer(*number, locale),
            None => number.to_string(),
        },
        MessageArgument::Decimal(number) => match locale {
            Some(locale) => format_decimal(number, locale),
            None => number.clone(),
        },
    }
}

fn plural_category(value: i64, locale: &Locale) -> &'static str {
    let n = value.unsigned_abs();
    match locale.primary_language() {
        "en" => {
            if n == 1 {
                "one"
            } else {
                "other"
            }
        }
        "fr" => {
            if n == 0 || n == 1 {
                "one"
            } else {
                "other"
            }
        }
        "ar" => match n {
            0 => "zero",
            1 => "one",
            2 => "two",
            _ if n % 100 >= 3 && n % 100 <= 10 => "few",
            _ if n % 100 >= 11 && n % 100 <= 99 => "many",
            _ => "other",
        },
        _ => "other",
    }
}

/// Formats an integer with grouping for English, French, and Arabic locales.
/// Other locale families use ASCII decimal digits without grouping.
fn exact_integer_from_decimal(value: &str) -> Option<i64> {
    let exponent_index = value.find(['e', 'E']);
    let (mantissa, exponent_text) = match exponent_index {
        Some(index) => (&value[..index], Some(&value[index + 1..])),
        None => (value, None),
    };
    let (negative, mantissa) = if let Some(value) = mantissa.strip_prefix('-') {
        (true, value)
    } else {
        (false, mantissa.strip_prefix('+').unwrap_or(mantissa))
    };
    let (integer, fraction) = mantissa.split_once('.').unwrap_or((mantissa, ""));
    if !integer.bytes().all(|digit| digit.is_ascii_digit())
        || !fraction.bytes().all(|digit| digit.is_ascii_digit())
        || integer.is_empty() && fraction.is_empty()
    {
        return None;
    }

    let digits = format!("{integer}{fraction}");
    if digits.is_empty() || !digits.bytes().all(|digit| digit.is_ascii_digit()) {
        return None;
    }
    if digits.bytes().all(|digit| digit == b'0') {
        if exponent_text.is_some_and(|text| {
            let digits = text.strip_prefix(['+', '-']).unwrap_or(text);
            digits.is_empty() || !digits.bytes().all(|digit| digit.is_ascii_digit())
        }) {
            return None;
        }
        return Some(0);
    }
    let exponent = exponent_text.map_or(Some(0), |text| text.parse::<i64>().ok())?;
    let shift = exponent.checked_sub(i64::try_from(fraction.len()).ok()?)?;
    let significant = if shift < 0 {
        let trailing_zeros = usize::try_from(shift.unsigned_abs()).ok()?;
        if trailing_zeros > digits.len()
            || !digits[digits.len() - trailing_zeros..]
                .bytes()
                .all(|digit| digit == b'0')
        {
            return None;
        }
        &digits[..digits.len() - trailing_zeros]
    } else {
        digits.as_str()
    };
    let significant = significant.trim_start_matches('0');
    if significant.is_empty() {
        return Some(0);
    }

    let appended_zeros = usize::try_from(shift.max(0)).ok()?;
    if significant.len().checked_add(appended_zeros)? > 19 {
        return None;
    }
    let mut integer = significant.to_owned();
    integer.extend(std::iter::repeat_n('0', appended_zeros));
    if negative {
        integer.insert(0, '-');
    }
    integer.parse().ok()
}

fn decimal_integer_operands(value: &str) -> Option<(u8, u8)> {
    let exponent_index = value.find(['e', 'E']);
    let (mantissa, exponent_text) = match exponent_index {
        Some(index) => (&value[..index], Some(&value[index + 1..])),
        None => (value, None),
    };
    let mantissa = mantissa
        .strip_prefix('-')
        .or_else(|| mantissa.strip_prefix('+'))
        .unwrap_or(mantissa);
    let (integer, fraction) = mantissa.split_once('.').unwrap_or((mantissa, ""));
    if !integer.bytes().all(|digit| digit.is_ascii_digit())
        || !fraction.bytes().all(|digit| digit.is_ascii_digit())
        || integer.is_empty() && fraction.is_empty()
    {
        return None;
    }

    let digits = format!("{integer}{fraction}");
    if digits.bytes().all(|digit| digit == b'0') {
        return Some((0, 0));
    }
    let exponent = exponent_text.map_or(Some(0), |text| text.parse::<i64>().ok())?;
    let fractional_digits = i64::try_from(fraction.len()).ok()?;
    let shift = exponent.checked_sub(fractional_digits)?;
    let significant_end = if shift < 0 {
        let trailing_zeros = usize::try_from(shift.unsigned_abs()).ok()?;
        if trailing_zeros > digits.len() {
            return digits.bytes().all(|digit| digit == b'0').then_some((0, 0));
        }
        let significant_end = digits.len() - trailing_zeros;
        if !digits[significant_end..].bytes().all(|digit| digit == b'0') {
            return None;
        }
        significant_end
    } else {
        digits.len()
    };
    let digits = digits[..significant_end].trim_start_matches('0');
    if digits.is_empty() {
        return Some((0, 0));
    }

    let shift = shift.max(0);
    let modulo_100 = match shift {
        0 => digits
            .as_bytes()
            .iter()
            .rev()
            .take(2)
            .rev()
            .fold(0_u8, |value, digit| value * 10 + (digit - b'0')),
        1 => (digits.as_bytes().last()? - b'0') * 10,
        _ => 0,
    };
    let small_value = if shift == 0 && digits.len() == 1 {
        digits.as_bytes()[0] - b'0'
    } else {
        u8::MAX
    };
    Some((small_value, modulo_100))
}

fn decimal_integer_part_is_zero_or_one(value: &str) -> bool {
    let exponent_index = value.find(['e', 'E']);
    let (mantissa, exponent) = match exponent_index {
        Some(index) => {
            let exponent_text = &value[index + 1..];
            let Ok(exponent) = exponent_text.parse::<i64>() else {
                let nonzero = value[..index]
                    .bytes()
                    .any(|digit| digit.is_ascii_digit() && digit != b'0');
                return !nonzero || exponent_text.starts_with('-');
            };
            (&value[..index], exponent)
        }
        None => (value, 0),
    };
    let mantissa = mantissa
        .strip_prefix('-')
        .or_else(|| mantissa.strip_prefix('+'))
        .unwrap_or(mantissa);
    let (integer, fraction) = mantissa.split_once('.').unwrap_or((mantissa, ""));
    let digits = format!("{integer}{fraction}");
    if digits.bytes().all(|digit| digit == b'0') {
        return true;
    }
    let Ok(integer_digits) = i64::try_from(integer.len()) else {
        return false;
    };
    let Some(decimal_point) = integer_digits.checked_add(exponent) else {
        return exponent < 0;
    };
    if decimal_point <= 0 {
        return true;
    }
    let Ok(decimal_point) = usize::try_from(decimal_point) else {
        return false;
    };
    if decimal_point > digits.len() {
        return false;
    }
    matches!(digits[..decimal_point].trim_start_matches('0'), "" | "1")
}

fn plural_category_decimal(value: &str, locale: &Locale) -> &'static str {
    if !is_decimal_literal(value) {
        return "other";
    }

    match locale.primary_language() {
        "en" => {
            let Ok(number) = value.parse::<f64>() else {
                return "other";
            };
            if !number.is_finite() {
                return "other";
            }
            let number = number.abs();
            if number == 1.0 && !value.contains('.') {
                "one"
            } else {
                "other"
            }
        }
        "fr" => {
            if decimal_integer_part_is_zero_or_one(value) {
                "one"
            } else {
                "other"
            }
        }
        "ar" => {
            let Some((small_value, modulo_100)) = decimal_integer_operands(value) else {
                return "other";
            };
            match small_value {
                0 => "zero",
                1 => "one",
                2 => "two",
                _ if (3..=10).contains(&modulo_100) => "few",
                _ if (11..=99).contains(&modulo_100) => "many",
                _ => "other",
            }
        }
        _ => "other",
    }
}

fn ordinal_category(value: i64, locale: &Locale) -> &'static str {
    let number = value.unsigned_abs();
    match locale.primary_language() {
        "en" => {
            if (11..=13).contains(&(number % 100)) {
                return "other";
            }
            match number % 10 {
                1 => "one",
                2 => "two",
                3 => "few",
                _ => "other",
            }
        }
        "fr" if number == 1 => "one",
        _ => "other",
    }
}

fn ordinal_category_decimal(value: &str, locale: &Locale) -> &'static str {
    let Some((small_value, modulo_100)) = decimal_integer_operands(value) else {
        return "other";
    };
    match locale.primary_language() {
        "en" => {
            if (11..=13).contains(&modulo_100) {
                return "other";
            }
            match modulo_100 % 10 {
                1 => "one",
                2 => "two",
                3 => "few",
                _ => "other",
            }
        }
        "fr" if small_value == 1 => "one",
        _ => "other",
    }
}

fn arabic_latin_symbols(locale: &Locale) -> (char, char) {
    thread_local! {
        static LAST: std::cell::RefCell<Option<(String, (char, char))>> = const { std::cell::RefCell::new(None) };
    }
    LAST.with(|cache| {
        if let Some((key, symbols)) = cache.borrow().as_ref()
            && key == locale.as_str()
        {
            return *symbols;
        }
        use writeable::Writeable;
        let symbols = locale
            .as_str()
            .parse::<icu_locale_core::Locale>()
            .ok()
            .and_then(|locale| {
                icu_decimal::DecimalFormatter::try_new(locale.into(), Default::default()).ok()
            })
            .map(|formatter| {
                formatter
                    .format(&"1234.5".parse().expect("valid decimal sample"))
                    .write_to_string()
                    .into_owned()
            })
            .and_then(|sample| {
                let mut symbols = sample.chars().filter(|ch| !ch.is_ascii_digit());
                match (symbols.next(), symbols.next(), symbols.next()) {
                    (Some(group), Some(decimal), None) => Some((group, decimal)),
                    _ => None,
                }
            })
            .unwrap_or((',', '.'));
        *cache.borrow_mut() = Some((locale.as_str().to_owned(), symbols));
        symbols
    })
}

pub fn format_integer(value: i64, locale: &Locale) -> String {
    let language = locale.primary_language();
    let (separator, default_arabic_digits) = match language {
        "fr" => ('\u{202f}', false),
        "ar" if explicit_script(locale) == Some("Latn") => (',', false),
        "ar" => ('\u{066c}', true),
        "en" => (',', false),
        _ => return value.to_string(),
    };
    let arabic_digits = match numbering_system_override(locale) {
        Some("latn") => false,
        Some("arab") => true,
        _ => default_arabic_digits,
    };
    let separator = match numbering_system_override(locale) {
        Some("arab") => '\u{066c}',
        Some("latn") if language == "ar" => arabic_latin_symbols(locale).0,
        _ => separator,
    };

    let digits = value.unsigned_abs().to_string();
    let mut grouped = String::with_capacity(digits.len() + digits.len() / 3);
    let indian_grouping = uses_indian_grouping(locale);
    if value < 0 {
        if let Some(mark) = sign_directional_mark(locale, arabic_digits) {
            grouped.push(mark);
        }
        grouped.push('-');
    }
    for (index, digit) in digits.chars().enumerate() {
        if grouping_separator_before(index, digits.len(), indian_grouping) {
            grouped.push(separator);
        }
        grouped.push(if arabic_digits {
            char::from_u32('٠' as u32 + digit as u32 - '0' as u32).expect("ASCII digit offset")
        } else {
            digit
        });
    }
    grouped
}

/// Formats a finite decimal argument with locale decimal and grouping
/// separators. `f32` and `f64` conversions use their shortest round-trip text;
/// no rounding is applied. Other locales retain ASCII decimal formatting.
pub fn format_decimal(value: &str, locale: &Locale) -> String {
    if !is_decimal_literal(value) {
        return value.to_owned();
    }

    let raw = value;
    let (group_separator, decimal_separator, default_arabic_digits) =
        match locale.primary_language() {
            "fr" => ('\u{202f}', ',', false),
            "ar" if explicit_script(locale) == Some("Latn") => (',', '.', false),
            "ar" => ('\u{066c}', '\u{066b}', true),
            "en" => (',', '.', false),
            _ => return raw.to_owned(),
        };
    let arabic_digits = match numbering_system_override(locale) {
        Some("latn") => false,
        Some("arab") => true,
        _ => default_arabic_digits,
    };
    let (group_separator, decimal_separator) = match numbering_system_override(locale) {
        Some("arab") => ('\u{066c}', '\u{066b}'),
        Some("latn") if locale.primary_language() == "ar" => arabic_latin_symbols(locale),
        _ => (group_separator, decimal_separator),
    };

    let exponent_index = raw.find(|character| matches!(character, 'e' | 'E'));
    let (mantissa, exponent) = match exponent_index {
        Some(index) => (&raw[..index], &raw[index..]),
        None => (raw, ""),
    };
    let (sign, unsigned) = match mantissa.strip_prefix('-') {
        Some(unsigned) => (Some('-'), unsigned),
        None => match mantissa.strip_prefix('+') {
            Some(unsigned) => (Some('+'), unsigned),
            None => (None, mantissa),
        },
    };
    let (integer, fraction) = unsigned
        .split_once('.')
        .map_or((unsigned, None), |(integer, fraction)| {
            (integer, Some(fraction))
        });
    let integer = integer.trim_start_matches('0');
    let integer = if integer.is_empty() { "0" } else { integer };
    let indian_grouping = uses_indian_grouping(locale);

    let mut output = String::with_capacity(raw.len() + raw.len() / 3);
    if let Some(sign) = sign {
        if let Some(mark) = sign_directional_mark(locale, arabic_digits) {
            output.push(mark);
        }
        output.push(sign);
    }
    for (index, digit) in integer.chars().enumerate() {
        if grouping_separator_before(index, integer.len(), indian_grouping) {
            output.push(group_separator);
        }
        output.push(localize_decimal_digit(digit, arabic_digits));
    }
    if let Some(fraction) = fraction {
        output.push(decimal_separator);
        output.extend(
            fraction
                .chars()
                .map(|digit| localize_decimal_digit(digit, arabic_digits)),
        );
    }
    for (index, digit) in exponent.chars().enumerate() {
        if index == 1 && matches!(digit, '+' | '-') {
            if let Some(mark) = sign_directional_mark(locale, arabic_digits) {
                output.push(mark);
            }
        }
        output.push(localize_decimal_digit(digit, arabic_digits));
    }
    output
}

fn uses_indian_grouping(locale: &Locale) -> bool {
    if locale.primary_language() != "en" {
        return false;
    }

    if let Some(region_override) = region_override(locale) {
        return region_override == "inzzzz";
    }

    locale
        .without_extensions()
        .subtags()
        .any(|subtag| subtag == "IN")
}

fn region_override(locale: &Locale) -> Option<&str> {
    let subtags = locale.subtags().collect::<Vec<_>>();
    let unicode_extension = subtags
        .iter()
        .position(|subtag| subtag.len() == 1 && *subtag == "u")?;
    let mut extension_subtags = subtags[unicode_extension + 1..]
        .iter()
        .take_while(|subtag| subtag.len() != 1);
    while let Some(subtag) = extension_subtags.next() {
        if *subtag == "rg" {
            return extension_subtags.next().copied().filter(|value| {
                value.ends_with("zzzz")
                    && ((value.len() == 6
                        && value[..2].bytes().all(|byte| byte.is_ascii_lowercase()))
                        || (value.len() == 7
                            && value[..3].bytes().all(|byte| byte.is_ascii_digit())))
            });
        }
    }
    None
}

fn grouping_separator_before(index: usize, digit_count: usize, indian: bool) -> bool {
    let remaining = digit_count - index;
    index != 0
        && if indian {
            remaining >= 3 && (remaining - 3) % 2 == 0
        } else {
            remaining % 3 == 0
        }
}

fn is_decimal_literal(value: &str) -> bool {
    let unsigned = value
        .strip_prefix('-')
        .or_else(|| value.strip_prefix('+'))
        .unwrap_or(value);
    let (mantissa, exponent) = unsigned
        .split_once(['e', 'E'])
        .map_or((unsigned, None), |(mantissa, exponent)| {
            (mantissa, Some(exponent))
        });
    if unsigned.contains(['e', 'E']) && unsigned.matches(['e', 'E']).count() != 1 {
        return false;
    }
    if let Some(exponent) = exponent {
        let exponent = exponent
            .strip_prefix('-')
            .or_else(|| exponent.strip_prefix('+'))
            .unwrap_or(exponent);
        if exponent.is_empty() || !exponent.bytes().all(|byte| byte.is_ascii_digit()) {
            return false;
        }
    }

    let (integer, fraction) = mantissa.split_once('.').unwrap_or((mantissa, ""));
    (!integer.is_empty() || !fraction.is_empty())
        && integer.bytes().all(|byte| byte.is_ascii_digit())
        && fraction.bytes().all(|byte| byte.is_ascii_digit())
        && mantissa.matches('.').count() <= 1
}

fn localize_decimal_digit(digit: char, arabic_digits: bool) -> char {
    if arabic_digits && digit.is_ascii_digit() {
        char::from_u32('٠' as u32 + digit as u32 - '0' as u32).expect("ASCII digit offset")
    } else {
        digit
    }
}

fn sign_directional_mark(locale: &Locale, arabic_digits: bool) -> Option<char> {
    if arabic_digits {
        Some('\u{061c}')
    } else if locale.primary_language() == "ar" {
        Some('\u{200e}')
    } else {
        None
    }
}

fn numbering_system_override(locale: &Locale) -> Option<&str> {
    let subtags = locale.subtags().collect::<Vec<_>>();
    let unicode_extension = subtags
        .iter()
        .position(|subtag| subtag.len() == 1 && *subtag == "u")?;
    let mut extension_subtags = subtags[unicode_extension + 1..]
        .iter()
        .take_while(|subtag| subtag.len() != 1);
    while let Some(subtag) = extension_subtags.next() {
        if *subtag == "nu" {
            return extension_subtags
                .next()
                .filter(|value| value.len() >= 3)
                .copied();
        }
    }
    None
}

fn valid_argument_name(name: &str) -> bool {
    let mut bytes = name.bytes();
    bytes.next().is_some_and(|byte| byte.is_ascii_alphabetic())
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b'-'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interpolates_named_text_and_integer_arguments() {
        let arguments = MessageArguments::new()
            .with("name", "Ada")
            .with("count", 3_i32);

        assert_eq!(
            interpolate("Hello, {name}: {count} items", &arguments).unwrap(),
            "Hello, Ada: 3 items"
        );
    }

    #[test]
    fn ignores_extra_interpolation_arguments() {
        let arguments = MessageArguments::new().with("unused", "ignored");

        assert_eq!(
            interpolate("No placeholders", &arguments).unwrap(),
            "No placeholders"
        );
    }

    #[test]
    fn supports_escaped_braces_and_repeated_placeholders() {
        let arguments = MessageArguments::new().with("name", "RustUI");

        assert_eq!(
            interpolate("{{{name}}} / {name}", &arguments).unwrap(),
            "{RustUI} / RustUI"
        );
    }

    #[test]
    fn supports_escaped_closing_braces_inside_expression_bodies() {
        let arguments = MessageArguments::new().with("count", 1);
        assert_eq!(
            interpolate(r"{count, plural, other {literal \} closing}}", &arguments).unwrap(),
            "literal } closing"
        );
    }

    #[test]
    fn rejects_missing_malformed_and_unmatched_placeholders() {
        let arguments = MessageArguments::new();
        assert_eq!(
            interpolate("Hi, {name}", &arguments),
            Err(InterpolationError::MissingArgument("name".into()))
        );
        assert!(matches!(
            interpolate("{bad name}", &arguments),
            Err(InterpolationError::InvalidPlaceholder(_))
        ));
        assert_eq!(
            interpolate("{name", &arguments),
            Err(InterpolationError::UnmatchedOpeningBrace)
        );
        assert_eq!(
            interpolate("name}", &arguments),
            Err(InterpolationError::UnmatchedClosingBrace)
        );
    }

    #[test]
    fn formats_integers_for_supported_locale_families() {
        assert_eq!(
            format_integer(1_234_567, &Locale::parse("en-US").unwrap()),
            "1,234,567"
        );
        assert_eq!(
            format_integer(-1_234, &Locale::parse("fr-CA").unwrap()),
            "-1\u{202f}234"
        );
        assert_eq!(
            format_integer(1_234, &Locale::parse("ar-EG").unwrap()),
            "١\u{066c}٢٣٤"
        );
        assert_eq!(
            format_integer(-1_234, &Locale::parse("ar-EG").unwrap()),
            "\u{061c}-١\u{066c}٢٣٤"
        );
        assert_eq!(
            format_integer(i64::MIN, &Locale::parse("en").unwrap()),
            "-9,223,372,036,854,775,808"
        );
        assert_eq!(format_integer(1_234, &Locale::parse("ja").unwrap()), "1234");
        assert_eq!(
            format_integer(1_234, &Locale::parse("ar-EG-u-nu-latn").unwrap()),
            "1,234"
        );
        assert_eq!(
            format_integer(1_234, &Locale::parse("en-US-u-nu-arab").unwrap()),
            "١٬٢٣٤"
        );
        assert_eq!(
            format_integer(
                1_234,
                &Locale::parse("en-US-u-ca-gregory-x-nu-arab").unwrap()
            ),
            "1,234"
        );
    }

    #[test]
    fn numbering_override_changes_decimal_and_grouping_symbols() {
        let arabic_latin = Locale::parse("ar-EG-u-nu-latn").unwrap();
        let english_arabic = Locale::parse("en-US-u-nu-arab").unwrap();
        let french_arabic = Locale::parse("fr-FR-u-nu-arab").unwrap();
        assert_eq!(format_integer(1_234, &arabic_latin), "1,234");
        assert_eq!(format_decimal("1234.5", &arabic_latin), "1,234.5");
        for locale in [&english_arabic, &french_arabic] {
            assert_eq!(format_integer(1_234, locale), "١٬٢٣٤");
            assert_eq!(format_decimal("1234.5", locale), "١٬٢٣٤٫٥");
        }
    }

    #[test]
    fn moroccan_arabic_latin_number_symbols_match_icu() {
        let locale = Locale::parse("ar-MA-u-nu-latn").unwrap();
        let formatter = crate::locale_runtime::LocaleFormatter::try_new(&locale).unwrap();
        assert_eq!(formatter.format_decimal("1234.50").unwrap(), "1.234,50");
        assert_eq!(
            format_integer(1_234, &locale),
            formatter.format_integer(1_234)
        );
        assert_eq!(
            format_decimal("1234.50", &locale),
            formatter.format_decimal("1234.50").unwrap()
        );
        let egypt = Locale::parse("ar-EG-u-nu-latn").unwrap();
        assert_eq!(format_decimal("1234.5", &egypt), "1,234.5");
        assert_eq!(format_decimal("1234.5", &locale), "1.234,5");
        let unknown_calendar = Locale::parse("ar-MA-u-ca-unknown-nu-latn").unwrap();
        assert_eq!(format_decimal("1234.5", &unknown_calendar), "1.234,5");
        assert_eq!(format_decimal("1234.5e2", &locale), "1.234,5e2");
    }

    #[test]
    fn explicit_latin_script_uses_latin_symbols_for_arabic_numbers() {
        let locale = Locale::parse("ar-Latn").unwrap();
        assert_eq!(format_integer(-1_234, &locale), "\u{200e}-1,234");
        assert_eq!(format_decimal("-1234.5", &locale), "\u{200e}-1,234.5");
    }

    #[test]
    fn formats_english_indian_integers_and_decimals_with_indian_grouping() {
        let locale = Locale::parse("en-IN").unwrap();

        assert_eq!(format_integer(123_456, &locale), "1,23,456");
        assert_eq!(format_integer(12_345_678, &locale), "1,23,45,678");
        assert_eq!(format_decimal("12345678.5", &locale), "1,23,45,678.5");

        let indian_region_override = Locale::parse("en-US-u-rg-inzzzz").unwrap();
        assert_eq!(
            format_integer(12_345_678, &indian_region_override),
            "1,23,45,678"
        );
        assert_eq!(
            format_decimal("12345678.5", &indian_region_override),
            "1,23,45,678.5"
        );

        let us_region_override = Locale::parse("en-IN-u-rg-uszzzz").unwrap();
        assert_eq!(
            format_integer(12_345_678, &us_region_override),
            "12,345,678"
        );
        assert_eq!(
            format_decimal("12345678.5", &us_region_override),
            "12,345,678.5"
        );

        let private_use = Locale::parse("en-x-in").unwrap();
        assert_eq!(format_integer(12_345_678, &private_use), "12,345,678");
        assert_eq!(format_decimal("12345678.5", &private_use), "12,345,678.5");
    }

    #[test]
    fn formats_decimal_arguments_with_locale_separators_and_digits() {
        assert_eq!(
            format_decimal("12345.678", &Locale::parse("en-US").unwrap()),
            "12,345.678"
        );
        assert_eq!(
            format_decimal("1234.5", &Locale::parse("ja").unwrap()),
            "1234.5"
        );
        assert_eq!(
            format_decimal("-1234.5", &Locale::parse("fr-CA").unwrap()),
            "-1\u{202f}234,5"
        );
        assert_eq!(
            format_decimal("-1234.5", &Locale::parse("ar-EG").unwrap()),
            "\u{061c}-١\u{066c}٢٣٤\u{066b}٥"
        );
        assert_eq!(
            format_decimal("0.005", &Locale::parse("ar-EG").unwrap()),
            "٠\u{066b}٠٠٥"
        );
        assert_eq!(
            format_decimal("1234.5", &Locale::parse("fr-CA-u-nu-latn").unwrap()),
            "1\u{202f}234,5"
        );
        assert_eq!(
            format_decimal("1234.5", &Locale::parse("en-US-u-nu-arab").unwrap()),
            "١٬٢٣٤٫٥"
        );
        assert_eq!(
            format_decimal("1e+21", &Locale::parse("ar-EG").unwrap()),
            "١e\u{061c}+٢١"
        );
        assert_eq!(
            format_decimal("+1234.5", &Locale::parse("en-US").unwrap()),
            "+1,234.5"
        );
    }

    #[test]
    fn formats_finite_decimal_strings_outside_f64_range() {
        assert_eq!(
            format_decimal("1e400", &Locale::parse("ar-EG").unwrap()),
            "١e٤٠٠"
        );
        assert_eq!(
            format_decimal("1234.5e400", &Locale::parse("fr-FR").unwrap()),
            "1\u{202f}234,5e400"
        );
        assert_eq!(
            format_decimal("1234.5e-400", &Locale::parse("fr-FR").unwrap()),
            "1\u{202f}234,5e-400"
        );
    }

    #[test]
    fn formats_decimal_integer_part_without_grouping_leading_zeroes() {
        assert_eq!(
            format_decimal("0001234.50", &Locale::parse("en-US").unwrap()),
            "1,234.50"
        );
        assert_eq!(
            format_decimal("-0000.50", &Locale::parse("fr-FR").unwrap()),
            "-0,50"
        );
    }

    #[test]
    fn directional_sign_marks_follow_numbering_system_and_both_signs() {
        assert_eq!(
            format_decimal("+1234.5", &Locale::parse("ar-EG").unwrap()),
            "\u{061c}+١\u{066c}٢٣٤\u{066b}٥"
        );
        assert_eq!(
            format_decimal("+1234.5", &Locale::parse("ar-EG-u-nu-latn").unwrap()),
            "\u{200e}+1,234.5"
        );
        assert_eq!(
            format_integer(-1_234, &Locale::parse("en-US-u-nu-arab").unwrap()),
            "\u{061c}-١٬٢٣٤"
        );
    }

    #[test]
    fn directional_sign_marks_also_cover_scientific_exponent_signs() {
        assert_eq!(
            format_decimal("1e-21", &Locale::parse("ar-EG").unwrap()),
            "١e\u{061c}-٢١"
        );
        assert_eq!(
            format_decimal("1e+21", &Locale::parse("ar-EG-u-nu-latn").unwrap()),
            "1e\u{200e}+21"
        );
        assert_eq!(
            format_decimal("1e-21", &Locale::parse("en-US-u-nu-arab").unwrap()),
            "١e\u{061c}-٢١"
        );
        assert_eq!(
            format_decimal("1E+21", &Locale::parse("ar-EG").unwrap()),
            "١E\u{061c}+٢١"
        );
        assert_eq!(
            format_decimal("-1E+21", &Locale::parse("ar-EG").unwrap()),
            "\u{061c}-١E\u{061c}+٢١"
        );
    }

    #[test]
    fn localized_interpolation_formats_decimal_arguments_but_plain_interpolation_does_not() {
        let arguments = MessageArguments::new().with("amount", 1_234.5_f64);
        assert_eq!(
            interpolate_with_locale(
                "Amount: {amount}",
                &arguments,
                &Locale::parse("fr-FR").unwrap()
            )
            .unwrap(),
            "Amount: 1\u{202f}234,5"
        );
        assert_eq!(
            interpolate("Amount: {amount}", &arguments).unwrap(),
            "Amount: 1234.5"
        );
    }

    #[test]
    fn leaves_non_finite_decimal_values_ascii() {
        assert_eq!(
            format_decimal("inf", &Locale::parse("ar-EG").unwrap()),
            "inf"
        );
        assert_eq!(
            format_decimal("NaN", &Locale::parse("fr-FR").unwrap()),
            "NaN"
        );
    }

    #[test]
    fn locale_aware_interpolation_formats_only_integer_arguments() {
        let arguments = MessageArguments::new()
            .with("count", 12_345_i64)
            .with("label", "files");

        assert_eq!(
            interpolate_with_locale("{count} {label}", &arguments, &Locale::parse("fr").unwrap())
                .unwrap(),
            "12\u{202f}345 files"
        );
        assert_eq!(interpolate("{count}", &arguments).unwrap(), "12345");
    }
}

#[cfg(test)]
mod expression_tests {
    use super::*;

    fn locale(tag: &str) -> Locale {
        Locale::parse(tag).unwrap()
    }

    #[test]
    fn decimal_plural_operands_use_bounded_locale_cardinal_rules() {
        let message = "{count, plural, one {one} other {other}}";
        let amount = MessageArguments::new().with("count", 1.5_f64);

        assert_eq!(
            interpolate_with_locale(message, &amount, &Locale::parse("en").unwrap()),
            Ok("other".to_owned())
        );
        assert_eq!(
            interpolate_with_locale(message, &amount, &Locale::parse("fr").unwrap()),
            Ok("one".to_owned())
        );
        assert_eq!(
            interpolate_with_locale(message, &amount, &Locale::parse("ar").unwrap()),
            Ok("other".to_owned())
        );
    }

    #[test]
    fn malformed_decimal_arguments_fall_back_to_other_plural_category() {
        let template = "{count, plural, zero {zero} one {one} other {other}}";
        for (value, tag) in [("0e--", "ar"), ("1e-", "fr")] {
            let arguments =
                MessageArguments::new().with("count", MessageArgument::Decimal(value.to_owned()));
            assert_eq!(
                interpolate_with_locale(template, &arguments, &locale(tag)),
                Ok("other".to_owned()),
                "malformed decimal {value} must not select a numeric category"
            );
        }
    }

    #[test]
    fn french_decimal_plural_uses_exact_integer_part_near_two() {
        let message = "{count, plural, one {one} other {other}}";
        let just_below_two = MessageArguments::new().with(
            "count",
            MessageArgument::Decimal("1.9999999999999999999999999999999999999999999999".into()),
        );

        assert_eq!(
            interpolate_with_locale(message, &just_below_two, &locale("fr")),
            Ok("one".to_owned())
        );
    }

    #[test]
    fn decimal_plural_operand_keeps_integer_exact_branch_precedence() {
        let message = "{count, plural, =1 {exact} one {category} other {other}}";
        let amount = MessageArguments::new().with("count", 1.0_f64);

        assert_eq!(
            interpolate_with_locale(message, &amount, &Locale::parse("fr").unwrap()),
            Ok("exact".to_owned())
        );

        let maximum =
            MessageArguments::new().with("count", MessageArgument::Decimal(i64::MAX.to_string()));
        assert_eq!(
            interpolate_with_locale(
                "{count, plural, =9223372036854775807 {exact} other {other}}",
                &maximum,
                &Locale::parse("en").unwrap()
            ),
            Ok("exact".to_owned())
        );

        assert_eq!(
            interpolate_with_locale(
                message,
                &MessageArguments::new().with("count", MessageArgument::Decimal("100e-2".into()),),
                &Locale::parse("fr").unwrap()
            ),
            Ok("exact".to_owned())
        );
    }

    #[test]
    fn arabic_decimal_plural_categories_preserve_exact_integer_digits() {
        let template =
            "{count, plural, zero {zero} one {one} two {two} few {few} many {many} other {other}}";
        for (count, expected) in [
            ("10000000000000000000003", "few"),
            ("10000000000000000000011", "many"),
            ("10000000000000000000003.0", "few"),
            ("10000000000000000000003.5", "other"),
        ] {
            let arguments =
                MessageArguments::new().with("count", MessageArgument::Decimal(count.to_owned()));
            assert_eq!(
                interpolate_with_locale(template, &arguments, &locale("ar")).unwrap(),
                expected,
                "unexpected Arabic plural category for {count}"
            );
        }
    }

    #[test]
    fn arabic_decimal_zero_plural_ignores_unrepresentable_exponent() {
        let arguments = MessageArguments::new().with(
            "count",
            MessageArgument::Decimal("0e999999999999999999999".into()),
        );
        assert_eq!(
            interpolate_with_locale(
                "{count, plural, zero {zero} other {other}}",
                &arguments,
                &locale("ar")
            )
            .unwrap(),
            "zero"
        );
    }

    #[test]
    fn exact_zero_branch_precedes_category_for_unrepresentable_exponent() {
        let arguments = MessageArguments::new().with(
            "count",
            MessageArgument::Decimal("0e999999999999999999999".into()),
        );
        assert_eq!(
            interpolate_with_locale(
                "{count, plural, =0 {exact zero} zero {zero category} other {other}}",
                &arguments,
                &locale("ar")
            )
            .unwrap(),
            "exact zero"
        );
    }

    #[test]
    fn ordinal_expressions_use_bounded_english_french_and_arabic_rules() {
        let template = "{count, selectordinal, one {one} two {two} few {few} other {other}}";
        for (count, expected) in [
            (1, "one"),
            (2, "two"),
            (3, "few"),
            (4, "other"),
            (11, "other"),
            (12, "other"),
            (13, "other"),
            (21, "one"),
            (22, "two"),
            (23, "few"),
            (-1, "one"),
        ] {
            let arguments = MessageArguments::new().with("count", count);
            assert_eq!(
                interpolate_with_locale(template, &arguments, &locale("en")).unwrap(),
                expected,
                "unexpected English ordinal category for {count}"
            );
        }

        for (tag, count, expected) in [("fr", 1, "one"), ("fr", 2, "other"), ("ar", 1, "other")] {
            let arguments = MessageArguments::new().with("count", count);
            assert_eq!(
                interpolate_with_locale(template, &arguments, &locale(tag)).unwrap(),
                expected,
                "unexpected ordinal category for {tag} {count}"
            );
        }
    }

    #[test]
    fn ordinal_decimal_values_require_integral_operands_and_keep_exact_branches() {
        let template =
            "{count, selectordinal, =2 {exact} one {one} two {two} few {few} other {other}}";
        for (tag, count, expected) in [
            ("en", "1.0", "one"),
            ("en", "2.0", "exact"),
            ("en", "2e0", "exact"),
            ("en", "21.0", "one"),
            ("en", "21e0", "one"),
            ("en", "1.5", "other"),
            ("fr", "1.0", "one"),
            ("fr", "1.5", "other"),
            ("ar", "21.0", "other"),
        ] {
            let arguments =
                MessageArguments::new().with("count", MessageArgument::Decimal(count.to_owned()));
            assert_eq!(
                interpolate_with_locale(template, &arguments, &locale(tag)).unwrap(),
                expected,
                "unexpected ordinal category for {tag} {count}"
            );
        }
    }

    #[test]
    fn ordinal_pound_substitution_and_contract_kind_are_preserved() {
        let arguments = MessageArguments::new().with("count", 21);
        assert_eq!(
            interpolate_with_locale(
                "{count, selectordinal, one {#st} two {#nd} few {#rd} other {#th}}",
                &arguments,
                &locale("en")
            )
            .unwrap(),
            "21st"
        );

        let cardinal = message_contract("{count, plural, one {one} other {other}}").unwrap();
        let ordinal = message_contract("{count, selectordinal, one {one} other {other}}").unwrap();
        assert!(!cardinal.matches(&ordinal));
        assert!(matches!(
            interpolate(
                "{count, selectordinal, many {many} other {other}}",
                &MessageArguments::new().with("count", 4)
            ),
            Err(InterpolationError::InvalidMessage(_))
        ));
    }

    #[test]
    fn plural_rules_cover_english_french_and_arabic_integer_categories() {
        let template =
            "{count, plural, zero {zero} one {one} two {two} few {few} many {many} other {other}}";
        for (tag, count, expected) in [
            ("en", 0, "other"),
            ("en", 1, "one"),
            ("en", 2, "other"),
            ("fr", 0, "one"),
            ("fr", 1, "one"),
            ("fr", 2, "other"),
            ("ar", 0, "zero"),
            ("ar", 1, "one"),
            ("ar", 2, "two"),
            ("ar", 3, "few"),
            ("ar", 11, "many"),
            ("ar", 100, "other"),
        ] {
            let args = MessageArguments::new().with("count", count);
            assert_eq!(
                interpolate_with_locale(template, &args, &locale(tag)).unwrap(),
                expected
            );
        }
    }

    #[test]
    fn exact_plural_branch_precedes_category_and_nested_arguments_work() {
        let template = "{count, plural, =2 {Exactly two for {name}} one {One} other {{gender, select, female {Many for {name}} other {Many}}}}";
        let args = MessageArguments::new()
            .with("count", 2)
            .with("name", "Ada")
            .with("gender", "female");
        assert_eq!(
            interpolate_with_locale(template, &args, &locale("en")).unwrap(),
            "Exactly two for Ada"
        );
        let args = MessageArguments::new()
            .with("count", 5)
            .with("name", "Ada")
            .with("gender", "female");
        assert_eq!(
            interpolate_with_locale(template, &args, &locale("en")).unwrap(),
            "Many for Ada"
        );
    }

    #[test]
    fn text_select_uses_other_and_checks_argument_type() {
        let template = "{gender, select, female {She greets {name}} male {He greets {name}} other {They greet {name}}}";
        let args = MessageArguments::new()
            .with("gender", "female")
            .with("name", "Ada");
        assert_eq!(interpolate(template, &args).unwrap(), "She greets Ada");
        let args = MessageArguments::new()
            .with("gender", "unknown")
            .with("name", "Ada");
        assert_eq!(interpolate(template, &args).unwrap(), "They greet Ada");
        let args = MessageArguments::new().with("gender", 3);
        assert_eq!(
            interpolate(template, &args),
            Err(InterpolationError::WrongArgumentType("gender".into()))
        );
    }

    #[test]
    fn expressions_require_other_and_reject_malformed_or_missing_arguments() {
        assert!(matches!(
            interpolate("{n, plural, one {one}}", &MessageArguments::new()),
            Err(InterpolationError::InvalidMessage(_))
        ));
        assert!(matches!(
            interpolate("{g, select, other {x} other {y}}", &MessageArguments::new()),
            Err(InterpolationError::InvalidMessage(_))
        ));
        assert_eq!(
            interpolate("{n, plural, other {x}}", &MessageArguments::new()),
            Err(InterpolationError::MissingArgument("n".into()))
        );
    }
}

#[cfg(test)]
mod pound_substitution_tests {
    use super::*;

    fn locale(tag: &str) -> Locale {
        Locale::parse(tag).unwrap()
    }

    #[test]
    fn plural_pound_uses_active_locale_number_formatting() {
        let arguments = MessageArguments::new().with("count", 1_234_i64);
        let message = "{count, plural, other {# files}}";

        assert_eq!(
            interpolate_with_locale(message, &arguments, &locale("en-US")),
            Ok("1,234 files".to_owned())
        );
        assert_eq!(
            interpolate_with_locale(message, &arguments, &locale("fr-FR")),
            Ok("1\u{202f}234 files".to_owned())
        );
        assert_eq!(
            interpolate_with_locale(message, &arguments, &locale("ar-EG")),
            Ok("١\u{066c}٢٣٤ files".to_owned())
        );
        assert_eq!(
            interpolate_with_locale(message, &arguments, &locale("ar-EG-u-nu-latn")),
            Ok("1,234 files".to_owned())
        );
        assert_eq!(
            interpolate_with_locale(message, &arguments, &locale("en-US-u-nu-arab")),
            Ok("١٬٢٣٤ files".to_owned())
        );
    }

    #[test]
    fn decimal_plural_pound_uses_locale_decimal_formatting() {
        let arguments = MessageArguments::new().with("count", 1_234.5_f64);
        assert_eq!(
            interpolate_with_locale(
                "{count, plural, other {# files}}",
                &arguments,
                &locale("fr-FR")
            ),
            Ok("1\u{202f}234,5 files".to_owned())
        );
    }

    #[test]
    fn plural_pound_flows_through_select_and_nested_plural_scopes() {
        let select_arguments = MessageArguments::new()
            .with("count", 2)
            .with("kind", "files");
        assert_eq!(
            interpolate_with_locale(
                "{count, plural, other {{kind, select, files {# files} other {# item}}}}",
                &select_arguments,
                &locale("en")
            ),
            Ok("2 files".to_owned())
        );

        let nested_arguments = MessageArguments::new()
            .with("outer", 2)
            .with("inner", 1_234);
        assert_eq!(
            interpolate_with_locale(
                "{outer, plural, other {# / {inner, plural, other {#}}}}",
                &nested_arguments,
                &locale("fr")
            ),
            Ok("2 / 1\u{202f}234".to_owned())
        );
    }

    #[test]
    fn escaped_pound_and_pounds_outside_plural_branches_remain_literal() {
        let arguments = MessageArguments::new()
            .with("count", 7)
            .with("kind", "files");
        assert_eq!(
            interpolate_with_locale(
                r"{count, plural, other {\# #}} #",
                &arguments,
                &locale("en")
            ),
            Ok("# 7 #".to_owned())
        );
        assert_eq!(
            interpolate_with_locale("{kind, select, other {#}}", &arguments, &locale("en")),
            Ok("#".to_owned())
        );
    }

    #[test]
    fn ordinal_branches_support_escaped_and_substituted_pounds() {
        let arguments = MessageArguments::new().with("count", 7);
        assert_eq!(
            interpolate_with_locale(
                r"{count, selectordinal, other {\# #}} #",
                &arguments,
                &locale("en")
            ),
            Ok("# 7 #".to_owned())
        );
    }

    #[test]
    fn ordinal_pound_uses_active_locale_number_formatting() {
        let arguments = MessageArguments::new().with("count", 1_234_i64);
        let message = "{count, selectordinal, other {#}}";

        for (tag, expected) in [
            ("en-US", "1,234"),
            ("fr-FR", "1\u{202f}234"),
            ("ar-EG", "١\u{066c}٢٣٤"),
        ] {
            assert_eq!(
                interpolate_with_locale(message, &arguments, &locale(tag)),
                Ok(expected.to_owned()),
                "ordinal pound should use {tag} number formatting"
            );
        }
    }

    #[test]
    fn arabic_decimal_plural_operands_are_not_limited_by_f64_range() {
        let template =
            "{count, plural, zero {zero} one {one} two {two} few {few} many {many} other {other}}";
        let count = format!("1{}3", "0".repeat(400));
        assert!(!count.parse::<f64>().unwrap().is_finite());

        let arguments = MessageArguments::new().with("count", MessageArgument::Decimal(count));
        assert_eq!(
            interpolate_with_locale(template, &arguments, &locale("ar")).unwrap(),
            "few"
        );
    }
}
