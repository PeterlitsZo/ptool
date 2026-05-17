use crate::{Error, ErrorKind, Result};
use inquire::ui::{Color, RenderConfig, Styled};
use inquire::validator::Validation;
use inquire::{Confirm, InquireError, MultiSelect, Password, PasswordDisplayMode, Select, Text};
use regex::Regex;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PromptTextOptions {
    pub default: Option<String>,
    pub help: Option<String>,
    pub placeholder: Option<String>,
    pub required: bool,
    pub allow_empty: bool,
    pub trim: bool,
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub pattern: Option<String>,
}

impl Default for PromptTextOptions {
    fn default() -> Self {
        Self {
            default: None,
            help: None,
            placeholder: None,
            required: false,
            allow_empty: true,
            trim: false,
            min_length: None,
            max_length: None,
            pattern: None,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PromptConfirmOptions {
    pub default: Option<bool>,
    pub help: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PromptSelectOptions {
    pub help: Option<String>,
    pub page_size: Option<usize>,
    pub default_index: Option<usize>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PromptMultiSelectOptions {
    pub help: Option<String>,
    pub page_size: Option<usize>,
    pub default_indexes: Vec<usize>,
    pub min_selected: Option<usize>,
    pub max_selected: Option<usize>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PromptSecretOptions {
    pub help: Option<String>,
    pub required: bool,
    pub allow_empty: bool,
    pub confirm: bool,
    pub confirm_prompt: Option<String>,
    pub mismatch_message: Option<String>,
    pub display_toggle: bool,
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub pattern: Option<String>,
}

pub fn prompt_text(op: &str, prompt: &str, options: PromptTextOptions) -> Result<String> {
    ensure_non_empty_prompt(op, prompt)?;
    validate_text_options(op, &options)?;
    let compiled_pattern = compile_pattern(op, options.pattern.as_deref())?;

    let default_for_validator = options.default.clone();
    let trim_for_validator = options.trim;
    let required = options.required;
    let allow_empty = options.allow_empty;
    let min_length = options.min_length;
    let max_length = options.max_length;

    let mut text = Text::new(prompt).with_render_config(prompt_render_config());
    if let Some(default) = options.default.as_deref() {
        text = text.with_default(default);
    }
    if let Some(help) = options.help.as_deref() {
        text = text.with_help_message(help);
    }
    if let Some(placeholder) = options.placeholder.as_deref() {
        text = text.with_placeholder(placeholder);
    }

    text = text.with_validator(move |input: &str| {
        let candidate = normalize_input(input, trim_for_validator);
        if input.is_empty() && default_for_validator.is_some() {
            return Ok(Validation::Valid);
        }
        validate_text_candidate(
            candidate.as_deref(),
            required,
            allow_empty,
            min_length,
            max_length,
            compiled_pattern.as_ref(),
        )
    });

    let value = map_inquire_result(op, text.prompt())?;
    Ok(normalize_answer(value, options.trim))
}

pub fn prompt_confirm(op: &str, prompt: &str, options: PromptConfirmOptions) -> Result<bool> {
    ensure_non_empty_prompt(op, prompt)?;
    let mut confirm = Confirm::new(prompt).with_render_config(prompt_render_config());
    if let Some(default) = options.default {
        confirm = confirm.with_default(default);
    }
    if let Some(help) = options.help.as_deref() {
        confirm = confirm.with_help_message(help);
    }
    map_inquire_result(op, confirm.prompt())
}

pub fn prompt_select(
    op: &str,
    prompt: &str,
    items: Vec<PromptItem>,
    options: PromptSelectOptions,
) -> Result<String> {
    ensure_non_empty_prompt(op, prompt)?;
    validate_select_options(op, &items, &options)?;

    let default_index = options.default_index;
    let mut select = Select::new(prompt, items).with_render_config(prompt_render_config());
    if let Some(help) = options.help.as_deref() {
        select = select.with_help_message(help);
    }
    if let Some(page_size) = options.page_size {
        select = select.with_page_size(page_size);
    }
    if let Some(default_index) = default_index {
        select = select.with_starting_cursor(default_index);
    }

    let selected = map_inquire_result(op, select.prompt())?;
    Ok(selected.value)
}

pub fn prompt_multiselect(
    op: &str,
    prompt: &str,
    items: Vec<PromptItem>,
    options: PromptMultiSelectOptions,
) -> Result<Vec<String>> {
    ensure_non_empty_prompt(op, prompt)?;
    validate_multiselect_options(op, &items, &options)?;

    let min_selected = options.min_selected;
    let max_selected = options.max_selected;

    let mut select = MultiSelect::new(prompt, items).with_render_config(prompt_render_config());
    if let Some(help) = options.help.as_deref() {
        select = select.with_help_message(help);
    }
    if let Some(page_size) = options.page_size {
        select = select.with_page_size(page_size);
    }
    if !options.default_indexes.is_empty() {
        select = select.with_default(&options.default_indexes);
    }
    if let Some(default_index) = options.default_indexes.first().copied() {
        select = select.with_starting_cursor(default_index);
    }
    select = select.with_validator(
        move |selected: &[inquire::list_option::ListOption<&PromptItem>]| {
            let count = selected.len();
            if let Some(min_selected) = min_selected
                && count < min_selected
            {
                return Ok(Validation::Invalid(
                    format!("Select at least {min_selected} item(s).").into(),
                ));
            }
            if let Some(max_selected) = max_selected
                && count > max_selected
            {
                return Ok(Validation::Invalid(
                    format!("Select at most {max_selected} item(s).").into(),
                ));
            }
            Ok(Validation::Valid)
        },
    );

    let selected = map_inquire_result(op, select.prompt())?;
    Ok(selected.into_iter().map(|item| item.value).collect())
}

pub fn prompt_secret(op: &str, prompt: &str, options: PromptSecretOptions) -> Result<String> {
    ensure_non_empty_prompt(op, prompt)?;
    validate_secret_options(op, &options)?;
    let compiled_pattern = compile_pattern(op, options.pattern.as_deref())?;
    let required = options.required;
    let allow_empty = options.allow_empty;
    let min_length = options.min_length;
    let max_length = options.max_length;
    let rendered_confirm_prompt = options.confirm.then(|| {
        options
            .confirm_prompt
            .clone()
            .unwrap_or_else(|| "Confirmation:".to_string())
    });

    let mut password = Password::new(prompt)
        .with_render_config(prompt_render_config())
        .with_display_mode(PasswordDisplayMode::Masked)
        .without_confirmation();
    if let Some(help) = options.help.as_deref() {
        password = password.with_help_message(help);
    }
    if options.confirm {
        password = Password::new(prompt)
            .with_render_config(prompt_render_config())
            .with_display_mode(PasswordDisplayMode::Masked)
            .with_custom_confirmation_message(
                rendered_confirm_prompt
                    .as_deref()
                    .expect("confirmation prompt must exist when confirm is enabled"),
            );
        if let Some(help) = options.help.as_deref() {
            password = password.with_help_message(help);
        }
        if let Some(mismatch_message) = options.mismatch_message.as_deref() {
            password = password.with_custom_confirmation_error_message(mismatch_message);
        }
    }
    if options.display_toggle {
        password = password.with_display_toggle_enabled();
    }
    password = password.with_validator(move |input: &str| {
        validate_text_candidate(
            Some(input),
            required,
            allow_empty,
            min_length,
            max_length,
            compiled_pattern.as_ref(),
        )
    });

    map_inquire_result(op, password.prompt())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PromptItem {
    pub label: String,
    pub value: String,
}

impl std::fmt::Display for PromptItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.label)
    }
}

fn map_inquire_result<T>(op: &str, result: std::result::Result<T, InquireError>) -> Result<T> {
    result.map_err(|err| match err {
        InquireError::NotTTY | InquireError::IO(_) => Error::new(
            ErrorKind::NotInteractive,
            format!("{op} requires an interactive TTY"),
        )
        .with_op(op),
        InquireError::OperationCanceled | InquireError::OperationInterrupted => {
            Error::new(ErrorKind::Cancelled, format!("{op} cancelled by user")).with_op(op)
        }
        other => Error::new(ErrorKind::Prompt, format!("{op} failed: {other}")).with_op(op),
    })
}

fn ensure_non_empty_prompt(op: &str, prompt: &str) -> Result<()> {
    if prompt.is_empty() {
        return Err(Error::new(
            ErrorKind::InvalidArgs,
            format!("{op} `prompt` must not be empty"),
        )
        .with_op(op)
        .with_detail("`prompt` must not be empty"));
    }
    Ok(())
}

fn prompt_render_config() -> RenderConfig<'static> {
    RenderConfig::default()
        .with_prompt_prefix(Styled::new("· ?").with_fg(Color::LightGreen))
        .with_answered_prompt_prefix(Styled::new("· >").with_fg(Color::LightGreen))
}

fn validate_text_options(op: &str, options: &PromptTextOptions) -> Result<()> {
    validate_length_bounds(op, options.min_length, options.max_length)?;
    if options.required && options.allow_empty {
        return Err(invalid_prompt_option(
            op,
            "`required` and `allow_empty` cannot both be true",
        ));
    }
    Ok(())
}

fn validate_secret_options(op: &str, options: &PromptSecretOptions) -> Result<()> {
    validate_length_bounds(op, options.min_length, options.max_length)?;
    if options.required && options.allow_empty {
        return Err(invalid_prompt_option(
            op,
            "`required` and `allow_empty` cannot both be true",
        ));
    }
    Ok(())
}

fn validate_select_options(
    op: &str,
    items: &[PromptItem],
    options: &PromptSelectOptions,
) -> Result<()> {
    validate_items(op, items)?;
    validate_page_size(op, options.page_size)?;
    if let Some(default_index) = options.default_index
        && default_index >= items.len()
    {
        return Err(invalid_prompt_option(
            op,
            format!("`default_index` {default_index} is out of range"),
        ));
    }
    Ok(())
}

fn validate_multiselect_options(
    op: &str,
    items: &[PromptItem],
    options: &PromptMultiSelectOptions,
) -> Result<()> {
    validate_items(op, items)?;
    validate_page_size(op, options.page_size)?;
    validate_length_bounds(op, options.min_selected, options.max_selected)?;
    for index in &options.default_indexes {
        if *index >= items.len() {
            return Err(invalid_prompt_option(
                op,
                format!("`default_indexes` contains out-of-range index {index}"),
            ));
        }
    }
    Ok(())
}

fn validate_items(op: &str, items: &[PromptItem]) -> Result<()> {
    if items.is_empty() {
        return Err(Error::new(
            ErrorKind::InvalidArgs,
            format!("{op} `items` must not be empty"),
        )
        .with_op(op)
        .with_detail("`items` must not be empty"));
    }
    for item in items {
        if item.label.is_empty() {
            return Err(invalid_prompt_option(
                op,
                "`items` labels must not be empty",
            ));
        }
    }
    Ok(())
}

fn validate_page_size(op: &str, page_size: Option<usize>) -> Result<()> {
    if matches!(page_size, Some(0)) {
        return Err(invalid_prompt_option(
            op,
            "`page_size` must be greater than 0",
        ));
    }
    Ok(())
}

fn validate_length_bounds(op: &str, min: Option<usize>, max: Option<usize>) -> Result<()> {
    if let (Some(min), Some(max)) = (min, max)
        && min > max
    {
        return Err(invalid_prompt_option(
            op,
            format!(
                "`min_length`/`min_selected` {min} cannot be greater than `max_length`/`max_selected` {max}"
            ),
        ));
    }
    Ok(())
}

fn compile_pattern(op: &str, pattern: Option<&str>) -> Result<Option<Regex>> {
    let Some(pattern) = pattern else {
        return Ok(None);
    };
    Regex::new(pattern)
        .map(Some)
        .map_err(|err| invalid_prompt_option(op, format!("`pattern` must be a valid regex: {err}")))
}

fn validate_text_candidate(
    candidate: Option<&str>,
    required: bool,
    allow_empty: bool,
    min_length: Option<usize>,
    max_length: Option<usize>,
    pattern: Option<&Regex>,
) -> std::result::Result<Validation, inquire::CustomUserError> {
    let candidate = candidate.unwrap_or("");
    if candidate.is_empty() {
        if required || !allow_empty {
            return Ok(Validation::Invalid("A response is required.".into()));
        }
        return Ok(Validation::Valid);
    }

    let length = candidate.chars().count();
    if let Some(min_length) = min_length
        && length < min_length
    {
        return Ok(Validation::Invalid(
            format!("The length of the response should be at least {min_length}.").into(),
        ));
    }
    if let Some(max_length) = max_length
        && length > max_length
    {
        return Ok(Validation::Invalid(
            format!("The length of the response should be at most {max_length}.").into(),
        ));
    }
    if let Some(pattern) = pattern
        && !pattern.is_match(candidate)
    {
        return Ok(Validation::Invalid(
            "The response does not match the required pattern.".into(),
        ));
    }
    Ok(Validation::Valid)
}

fn normalize_input(input: &str, trim: bool) -> Option<String> {
    let normalized = if trim { input.trim() } else { input };
    Some(normalized.to_string())
}

fn normalize_answer(answer: String, trim: bool) -> String {
    if trim {
        answer.trim().to_string()
    } else {
        answer
    }
}

fn invalid_prompt_option(op: &str, detail: impl Into<String>) -> Error {
    let detail = detail.into();
    Error::new(ErrorKind::InvalidPromptOptions, format!("{op} {detail}"))
        .with_op(op)
        .with_detail(detail)
}
