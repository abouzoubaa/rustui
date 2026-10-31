//! Project Fluent evaluation. Resource-locale number formatting uses ICU4X.
//!
//! Fluent's language identifier has no Unicode extensions. Its NUMBER output
//! uses the resource's language/script/region, while `LocaleFormatter` accepts
//! full ICU locale preferences. Pass preformatted strings when `u-nu` matters.
//! Selectors use intl_pluralrules 7.0.2's source-declared CLDR 37 table, not
//! ICU4X's CLDR 49 formatting data. Its README's CLDR 36 claim is stale with
//! respect to the generated `src/rules.rs` CLDR_VERSION constant; no plural
//! parity with CLDR 49 is promised.
use crate::{Locale, MessageArgument, MessageArguments, MessageId};
use fluent_bundle::types::FluentNumber;
use fluent_bundle::{FluentArgs, FluentResource, FluentValue, concurrent::FluentBundle};
use fluent_syntax::ast::{
    CallArguments, Entry, Expression, InlineExpression, Pattern, PatternElement,
};
use icu_decimal::{
    DecimalFormatter,
    input::{Decimal, FloatPrecision},
};
use intl_memoizer::{Memoizable, concurrent::IntlLangMemoizer};
use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fmt,
    str::FromStr,
};
use unic_langid::LanguageIdentifier;
use writeable::Writeable;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FluentFormatError {
    InvalidLocale(String),
    InvalidResource(Vec<String>),
    UnsupportedNumberOption(String),
    UnsupportedFunction(String),
    InvalidNumber(String),
    InvalidArgument { name: String, reason: String },
    MissingMessage(String),
    Evaluation(Vec<String>),
}
impl fmt::Display for FluentFormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLocale(value) => write!(f, "unsupported Fluent locale `{value}`"),
            Self::InvalidResource(errors) => {
                write!(f, "invalid Fluent resource: {}", errors.join("; "))
            }
            Self::UnsupportedNumberOption(option) => {
                write!(f, "unsupported Fluent NUMBER option `{option}`")
            }
            Self::UnsupportedFunction(name) => write!(f, "unsupported Fluent function `{name}`"),
            Self::InvalidNumber(value) => write!(f, "invalid Fluent number `{value}`"),
            Self::InvalidArgument { name, reason } => {
                write!(f, "Fluent argument `{name}`: {reason}")
            }
            Self::MissingMessage(id) => write!(f, "missing Fluent message or attribute `{id}`"),
            Self::Evaluation(errors) => {
                write!(f, "Fluent evaluation failed: {}", errors.join("; "))
            }
        }
    }
}
impl Error for FluentFormatError {}

/// An ICU number formatter memoized by Fluent's existing locale memoizer.
struct FluentDecimalFormatter(DecimalFormatter);
impl Memoizable for FluentDecimalFormatter {
    type Args = ();
    type Error = String;
    fn construct(locale: LanguageIdentifier, _: ()) -> Result<Self, String> {
        let locale: icu_locale_core::Locale =
            locale.to_string().parse().map_err(|e| format!("{e}"))?;
        DecimalFormatter::try_new(locale.into(), Default::default())
            .map(Self)
            .map_err(|e| e.to_string())
    }
}
fn format_number(value: &FluentValue<'_>, memoizer: &IntlLangMemoizer) -> Option<String> {
    let FluentValue::Number(number) = value else {
        return None;
    };
    // Resource policy and argument conversion check finite values and precision
    // before evaluation. ICU RoundTrip keeps Fluent's f64 representation.
    let mut decimal = Decimal::try_from_f64(number.value, FloatPrecision::RoundTrip).ok()?;
    if let Some(digits) = number.options.minimum_fraction_digits {
        decimal.pad_end(-i16::try_from(digits).ok()?);
    }
    memoizer
        .with_try_get::<FluentDecimalFormatter, _, _>((), |formatter| {
            formatter.0.format(&decimal).write_to_string().into_owned()
        })
        .ok()
}

pub(crate) struct FluentCatalog {
    bundle: FluentBundle<FluentResource>,
    contracts: BTreeMap<String, Contract>,
}
impl fmt::Debug for FluentCatalog {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FluentCatalog")
            .field("locales", &self.bundle.locales)
            .finish_non_exhaustive()
    }
}
#[derive(Default)]
struct Contract {
    variables: BTreeSet<String>,
    references: BTreeSet<String>,
    terms: BTreeSet<String>,
}
impl FluentCatalog {
    pub(crate) fn parse(locale: &Locale, source: &str) -> Result<Self, FluentFormatError> {
        let resource_locale: LanguageIdentifier = locale
            .without_extensions()
            .as_str()
            .parse()
            .map_err(|_| FluentFormatError::InvalidLocale(locale.to_string()))?;
        // Preflight ICU so a formatter load failure never becomes Fluent's
        // invariant numeric fallback through the optional formatter hook.
        FluentDecimalFormatter::construct(resource_locale.clone(), ())
            .map_err(|error| FluentFormatError::InvalidResource(vec![error]))?;
        let resource = FluentResource::try_new(source.to_owned()).map_err(|(_, errors)| {
            FluentFormatError::InvalidResource(
                errors.into_iter().map(|error| error.to_string()).collect(),
            )
        })?;
        let mut contracts = BTreeMap::new();
        for entry in resource.entries() {
            match entry {
                Entry::Message(message) => {
                    if let Some(value) = &message.value {
                        insert_contract(&mut contracts, message.id.name.to_owned(), value)?;
                    }
                    for attribute in &message.attributes {
                        insert_contract(
                            &mut contracts,
                            format!("{}.{}", message.id.name, attribute.id.name),
                            &attribute.value,
                        )?;
                    }
                }
                Entry::Term(term) => {
                    insert_contract(&mut contracts, format!("-{}", term.id.name), &term.value)?;
                    for attribute in &term.attributes {
                        insert_contract(
                            &mut contracts,
                            format!("-{}.{}", term.id.name, attribute.id.name),
                            &attribute.value,
                        )?;
                    }
                }
                _ => {}
            }
        }
        let mut bundle = FluentBundle::new_concurrent(vec![resource_locale]);
        bundle.set_formatter(Some(format_number));
        bundle
            .add_builtins()
            .map_err(|e| FluentFormatError::InvalidResource(vec![e.to_string()]))?;
        bundle.add_resource(resource).map_err(|errors| {
            FluentFormatError::InvalidResource(errors.into_iter().map(|e| e.to_string()).collect())
        })?;
        Ok(Self { bundle, contracts })
    }
    pub(crate) fn message_ids(&self) -> impl Iterator<Item = MessageId> + '_ {
        self.contracts
            .keys()
            .filter(|id| !id.starts_with('-'))
            .map(|id| {
                MessageId::parse(id)
                    .expect("Fluent identifiers are valid RustUI message identifiers")
            })
    }
    pub(crate) fn contains(&self, identifier: &str) -> bool {
        !identifier.starts_with('-') && self.contracts.contains_key(identifier)
    }
    pub(crate) fn format(
        &self,
        identifier: &str,
        arguments: &MessageArguments,
    ) -> Result<String, FluentFormatError> {
        let (message_id, attribute) = identifier
            .split_once('.')
            .map_or((identifier, None), |(id, attribute)| (id, Some(attribute)));
        let message = self
            .bundle
            .get_message(message_id)
            .ok_or_else(|| FluentFormatError::MissingMessage(identifier.to_owned()))?;
        let pattern = match attribute {
            Some(attribute) => message.get_attribute(attribute).map(|a| a.value()),
            None => message.value(),
        }
        .ok_or_else(|| FluentFormatError::MissingMessage(identifier.to_owned()))?;
        let mut fluent_args = FluentArgs::new();
        for (name, value) in arguments.iter() {
            let number = match value {
                MessageArgument::Text(value) => {
                    fluent_args.set(name, value.as_str());
                    continue;
                }
                MessageArgument::Integer(value) => {
                    if !(-(1_i64 << 53)..=(1_i64 << 53)).contains(value) {
                        return Err(FluentFormatError::InvalidArgument { name: name.to_owned(), reason: "integer exceeds Fluent's exact f64 range; pass a preformatted string".to_owned() });
                    }
                    FluentNumber::new(*value as f64, Default::default())
                }
                MessageArgument::Decimal(value) => {
                    if value.contains(['e', 'E']) {
                        return Err(FluentFormatError::InvalidArgument {
                            name: name.to_owned(),
                            reason: "Fluent decimal arguments require plain decimal notation; pass a preformatted string for exponent notation".to_owned(),
                        });
                    }
                    // Fluent numbers are f64; exact decimal display is available
                    // through LocaleFormatter followed by a text argument.
                    value.parse::<FluentNumber>().map_err(|e| {
                        FluentFormatError::InvalidArgument {
                            name: name.to_owned(),
                            reason: e.to_string(),
                        }
                    })?
                }
            };
            validate_number(
                &number.value.to_string(),
                number.options.minimum_fraction_digits,
            )?;
            fluent_args.set(name, FluentValue::Number(number));
        }
        let mut errors = Vec::new();
        let output = self
            .bundle
            .format_pattern(pattern, Some(&fluent_args), &mut errors)
            .into_owned();
        if errors.is_empty() {
            Ok(output)
        } else {
            Err(FluentFormatError::Evaluation(
                errors.into_iter().map(|e| e.to_string()).collect(),
            ))
        }
    }
    /// Direct variables including message references. Term variables belong to
    /// their term argument scope, so call-site arguments are visited separately.
    pub(crate) fn arguments(&self, id: &str) -> Option<BTreeSet<String>> {
        let mut result = BTreeSet::new();
        let mut active = BTreeSet::new();
        let mut complete = BTreeSet::new();
        let mut pending = vec![(id, true, false)];
        while let Some((id, collect_variables, exiting)) = pending.pop() {
            if exiting {
                active.remove(id);
                complete.insert((id, collect_variables));
                continue;
            }
            if active.contains(id) {
                return None;
            }
            if complete.contains(&(id, collect_variables)) {
                continue;
            }
            let contract = self.contracts.get(id)?;
            active.insert(id);
            if collect_variables {
                result.extend(contract.variables.iter().cloned());
            }
            pending.push((id, collect_variables, true));
            pending.extend(
                contract
                    .references
                    .iter()
                    .map(|reference| (reference.as_str(), collect_variables, false)),
            );
            pending.extend(
                contract
                    .terms
                    .iter()
                    .map(|term| (term.as_str(), false, false)),
            );
        }
        Some(result)
    }
}
fn validate_number(value: &str, fraction_digits: Option<usize>) -> Result<(), FluentFormatError> {
    if !value.parse::<f64>().is_ok_and(f64::is_finite)
        || fraction_digits.is_some_and(|n| n > i16::MAX as usize)
    {
        return Err(FluentFormatError::InvalidNumber(value.to_owned()));
    }
    Ok(())
}
fn insert_contract(
    map: &mut BTreeMap<String, Contract>,
    id: String,
    pattern: &Pattern<&str>,
) -> Result<(), FluentFormatError> {
    let mut contract = Contract::default();
    visit_pattern(pattern, &mut contract)?;
    if map.insert(id.clone(), contract).is_some() {
        return Err(FluentFormatError::InvalidResource(vec![format!(
            "duplicate Fluent entry `{id}`"
        )]));
    }
    Ok(())
}
fn visit_pattern(
    pattern: &Pattern<&str>,
    contract: &mut Contract,
) -> Result<(), FluentFormatError> {
    for element in &pattern.elements {
        if let PatternElement::Placeable { expression } = element {
            visit_expression(expression, contract)?;
        }
    }
    Ok(())
}
fn visit_expression(
    expression: &Expression<&str>,
    contract: &mut Contract,
) -> Result<(), FluentFormatError> {
    match expression {
        Expression::Inline(value) => visit_inline(value, contract),
        Expression::Select { selector, variants } => {
            visit_inline(selector, contract)?;
            for variant in variants {
                visit_pattern(&variant.value, contract)?;
            }
            Ok(())
        }
    }
}
fn visit_arguments(
    arguments: &CallArguments<&str>,
    contract: &mut Contract,
) -> Result<(), FluentFormatError> {
    for value in &arguments.positional {
        visit_inline(value, contract)?;
    }
    for argument in &arguments.named {
        visit_inline(&argument.value, contract)?;
    }
    Ok(())
}
fn visit_inline(
    value: &InlineExpression<&str>,
    contract: &mut Contract,
) -> Result<(), FluentFormatError> {
    match value {
        InlineExpression::NumberLiteral { value } => {
            let parsed = FluentNumber::from_str(value)
                .map_err(|_| FluentFormatError::InvalidNumber((*value).to_owned()))?;
            validate_number(value, parsed.options.minimum_fraction_digits)?;
        }
        InlineExpression::VariableReference { id } => {
            contract.variables.insert(id.name.to_owned());
        }
        InlineExpression::MessageReference { id, attribute } => {
            contract.references.insert(attribute.as_ref().map_or_else(
                || id.name.to_owned(),
                |attribute| format!("{}.{}", id.name, attribute.name),
            ));
        }
        InlineExpression::TermReference {
            id,
            attribute,
            arguments,
        } => {
            contract.terms.insert(attribute.as_ref().map_or_else(
                || format!("-{}", id.name),
                |attribute| format!("-{}.{}", id.name, attribute.name),
            ));
            if let Some(arguments) = arguments {
                visit_arguments(arguments, contract)?;
            }
        }
        InlineExpression::FunctionReference { id, arguments } => {
            if id.name != "NUMBER" {
                return Err(FluentFormatError::UnsupportedFunction(id.name.to_owned()));
            }
            for argument in &arguments.named {
                let supported_type = argument.name.name == "type"
                    && matches!(
                        &argument.value,
                        InlineExpression::StringLiteral {
                            value: "cardinal" | "ordinal"
                        }
                    );
                if !supported_type {
                    return Err(FluentFormatError::UnsupportedNumberOption(
                        argument.name.name.to_owned(),
                    ));
                }
            }
            if arguments.positional.len() != 1 {
                return Err(FluentFormatError::InvalidResource(vec![
                    "NUMBER requires exactly one positional argument".to_owned(),
                ]));
            }
            visit_arguments(arguments, contract)?;
        }
        InlineExpression::Placeable { expression } => visit_expression(expression, contract)?,
        InlineExpression::StringLiteral { .. } => {}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fluent_icu_formatter_is_memoized_by_existing_locale_memoizer() {
        let memoizer = IntlLangMemoizer::new("fr-FR".parse().unwrap());
        let first = memoizer
            .with_try_get::<FluentDecimalFormatter, _, _>((), |formatter| {
                formatter as *const _ as usize
            })
            .unwrap();
        let second = memoizer
            .with_try_get::<FluentDecimalFormatter, _, _>((), |formatter| {
                formatter as *const _ as usize
            })
            .unwrap();
        assert_eq!(first, second);
        assert_eq!(
            format_number(
                &FluentValue::Number(FluentNumber::new(1234.5, Default::default())),
                &memoizer
            )
            .unwrap(),
            "1\u{202f}234,5"
        );
    }
}
