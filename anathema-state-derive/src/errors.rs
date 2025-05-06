use proc_macro2::Span;

pub fn report_missing_data(span: Span) -> syn::Error {
    syn::Error::new(span, "a string value is required")
}

pub fn report_empty_value(attr: &str, span: Span) -> syn::Error {
    syn::Error::new(span, format!("value cannot be empty for '{attr}'"))
}

pub fn report_exclusive_failure(attr: &str, span: Span) -> syn::Error {
    syn::Error::new(span, format!("'{attr}' cannot be used more than once"))
}

pub fn report_unique_failure(attr: &str, span: Span) -> syn::Error {
    syn::Error::new(
        span,
        format!(
            "'{attr}' cannot be used with other {} attributes",
            crate::DERIVE_NAMESPACE
        ),
    )
}

pub fn report_unknown_attribute<T: AsRef<str>>(
    available: impl IntoIterator<Item = T>,
    found: &str,
    span: Span,
) -> syn::Error {
    let known = available.into_iter().fold(String::new(), |mut a, c| {
        if !a.is_empty() {
            a.push_str(", ");
        }
        a.push_str(c.as_ref());
        a
    });
    let message = format!("unknown attribute: {found}, supported: {known}",);
    syn::Error::new(span, message)
}

pub fn reduce_errors<T>(
    okay: T,
    errors: impl IntoIterator<Item = syn::Error>,
) -> Result<T, syn::Error> {
    let Some(errors) = errors.into_iter().reduce(|mut left, right| {
        left.combine(right);
        left
    }) else {
        return Ok(okay);
    };
    Err(errors)
}
