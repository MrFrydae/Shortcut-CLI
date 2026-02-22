use std::error::Error;

#[derive(Debug, Clone)]
pub struct MemberChoice {
    pub display: String, // "Alice Smith (@alice)"
    pub value: String,   // "@alice"
}

#[derive(Debug, Clone)]
pub struct IdChoice {
    pub display: String,
    pub id: i64,
}

#[derive(Debug, Clone)]
pub struct UuidChoice {
    pub display: String,
    pub id: uuid::Uuid,
}

/// Abstraction over terminal prompts for testability.
pub trait Prompter {
    /// Prompt for required text input.
    fn prompt_text(&self, message: &str) -> Result<String, Box<dyn Error>>;

    /// Prompt for optional text input (empty string → None).
    fn prompt_optional_text(&self, message: &str) -> Result<Option<String>, Box<dyn Error>>;

    /// Prompt the user to select one item from a list. Returns the selected string.
    fn prompt_select(&self, message: &str, options: &[&str]) -> Result<String, Box<dyn Error>>;

    /// Prompt the user to optionally select one item. Returns None if skipped.
    fn prompt_optional_select(
        &self,
        message: &str,
        options: &[&str],
    ) -> Result<Option<String>, Box<dyn Error>>;

    /// Prompt for a comma-separated list of values. Returns empty vec if skipped.
    fn prompt_list(&self, message: &str) -> Result<Vec<String>, Box<dyn Error>>;

    /// Prompt the user to select zero or more items from a checkbox list.
    fn prompt_multi_select(
        &self,
        message: &str,
        choices: &[MemberChoice],
    ) -> Result<Vec<String>, Box<dyn Error>>;

    /// Prompt the user to optionally select an item by ID. Returns None if skipped.
    /// Falls back to `prompt_optional_i64` if choices is empty.
    fn prompt_optional_select_id(
        &self,
        message: &str,
        choices: &[IdChoice],
    ) -> Result<Option<i64>, Box<dyn Error>>;

    /// Prompt the user to select zero or more items by ID from a checkbox list.
    fn prompt_multi_select_id(
        &self,
        message: &str,
        choices: &[IdChoice],
    ) -> Result<Vec<i64>, Box<dyn Error>>;

    /// Prompt the user to optionally select an item by UUID. Returns None if skipped or empty.
    fn prompt_optional_select_uuid(
        &self,
        message: &str,
        choices: &[UuidChoice],
    ) -> Result<Option<uuid::Uuid>, Box<dyn Error>>;

    /// Prompt for an optional integer value.
    fn prompt_optional_i64(&self, message: &str) -> Result<Option<i64>, Box<dyn Error>>;

    /// Ask a yes/no confirmation question.
    fn confirm(&self, message: &str) -> Result<bool, Box<dyn Error>>;
}

// ── TerminalPrompter (production) ───────────────────────────────────

/// Production prompter backed by `dialoguer`.
pub struct TerminalPrompter;

impl Prompter for TerminalPrompter {
    fn prompt_text(&self, message: &str) -> Result<String, Box<dyn Error>> {
        let input: String = dialoguer::Input::new()
            .with_prompt(message)
            .interact_text()?;
        if input.trim().is_empty() {
            return Err("Value is required".into());
        }
        Ok(input)
    }

    fn prompt_optional_text(&self, message: &str) -> Result<Option<String>, Box<dyn Error>> {
        let input: String = dialoguer::Input::new()
            .with_prompt(format!("{message} (optional, enter to skip)"))
            .allow_empty(true)
            .interact_text()?;
        if input.trim().is_empty() {
            Ok(None)
        } else {
            Ok(Some(input))
        }
    }

    fn prompt_select(&self, message: &str, options: &[&str]) -> Result<String, Box<dyn Error>> {
        let idx = dialoguer::Select::new()
            .with_prompt(message)
            .items(options)
            .interact()?;
        Ok(options[idx].to_string())
    }

    fn prompt_optional_select(
        &self,
        message: &str,
        options: &[&str],
    ) -> Result<Option<String>, Box<dyn Error>> {
        let mut items: Vec<&str> = vec!["(skip)"];
        items.extend_from_slice(options);
        let idx = dialoguer::Select::new()
            .with_prompt(message)
            .items(&items)
            .default(0)
            .interact()?;
        if idx == 0 {
            Ok(None)
        } else {
            Ok(Some(items[idx].to_string()))
        }
    }

    fn prompt_optional_select_id(
        &self,
        message: &str,
        choices: &[IdChoice],
    ) -> Result<Option<i64>, Box<dyn Error>> {
        if choices.is_empty() {
            return self.prompt_optional_i64(message);
        }
        let mut items: Vec<&str> = vec!["(skip)"];
        items.extend(choices.iter().map(|c| c.display.as_str()));
        let idx = dialoguer::Select::new()
            .with_prompt(message)
            .items(&items)
            .default(0)
            .interact()?;
        if idx == 0 {
            Ok(None)
        } else {
            Ok(Some(choices[idx - 1].id))
        }
    }

    fn prompt_multi_select_id(
        &self,
        message: &str,
        choices: &[IdChoice],
    ) -> Result<Vec<i64>, Box<dyn Error>> {
        if choices.is_empty() {
            return Ok(vec![]);
        }
        let display: Vec<&str> = choices.iter().map(|c| c.display.as_str()).collect();
        let selected = dialoguer::MultiSelect::new()
            .with_prompt(format!("{message} (space to toggle, enter to confirm)"))
            .items(&display)
            .interact()?;
        Ok(selected.into_iter().map(|i| choices[i].id).collect())
    }

    fn prompt_optional_select_uuid(
        &self,
        message: &str,
        choices: &[UuidChoice],
    ) -> Result<Option<uuid::Uuid>, Box<dyn Error>> {
        if choices.is_empty() {
            return Ok(None);
        }
        let mut items: Vec<&str> = vec!["(skip)"];
        items.extend(choices.iter().map(|c| c.display.as_str()));
        let idx = dialoguer::Select::new()
            .with_prompt(message)
            .items(&items)
            .default(0)
            .interact()?;
        if idx == 0 {
            Ok(None)
        } else {
            Ok(Some(choices[idx - 1].id))
        }
    }

    fn prompt_list(&self, message: &str) -> Result<Vec<String>, Box<dyn Error>> {
        let input: String = dialoguer::Input::new()
            .with_prompt(format!("{message} (comma-separated, enter to skip)"))
            .allow_empty(true)
            .interact_text()?;
        if input.trim().is_empty() {
            Ok(vec![])
        } else {
            Ok(input.split(',').map(|s| s.trim().to_string()).collect())
        }
    }

    fn prompt_multi_select(
        &self,
        message: &str,
        choices: &[MemberChoice],
    ) -> Result<Vec<String>, Box<dyn Error>> {
        if choices.is_empty() {
            return Ok(vec![]);
        }
        let display: Vec<&str> = choices.iter().map(|c| c.display.as_str()).collect();
        let selected = dialoguer::MultiSelect::new()
            .with_prompt(format!("{message} (space to toggle, enter to confirm)"))
            .items(&display)
            .interact()?;
        Ok(selected
            .into_iter()
            .map(|i| choices[i].value.clone())
            .collect())
    }

    fn prompt_optional_i64(&self, message: &str) -> Result<Option<i64>, Box<dyn Error>> {
        let input: String = dialoguer::Input::new()
            .with_prompt(format!("{message} (optional, enter to skip)"))
            .allow_empty(true)
            .interact_text()?;
        if input.trim().is_empty() {
            Ok(None)
        } else {
            let val = input
                .trim()
                .parse::<i64>()
                .map_err(|_| format!("Invalid number: {input}"))?;
            Ok(Some(val))
        }
    }

    fn confirm(&self, message: &str) -> Result<bool, Box<dyn Error>> {
        Ok(dialoguer::Confirm::new()
            .with_prompt(message)
            .default(true)
            .interact()?)
    }
}

// ── MockPrompter (tests) ────────────────────────────────────────────

pub use mock::*;

mod mock {
    use super::*;
    use std::cell::RefCell;
    use std::collections::VecDeque;

    /// Typed answer variants for the mock prompter.
    pub enum MockAnswer {
        Text(String),
        OptionalText(Option<String>),
        Select(String),
        OptionalSelect(Option<String>),
        List(Vec<String>),
        MultiSelect(Vec<String>),
        MultiSelectId(Vec<i64>),
        OptionalI64(Option<i64>),
        OptionalUuid(Option<uuid::Uuid>),
        Confirm(bool),
    }

    /// A test prompter that returns pre-loaded answers.
    pub struct MockPrompter {
        answers: RefCell<VecDeque<MockAnswer>>,
    }

    impl MockPrompter {
        pub fn new(answers: Vec<MockAnswer>) -> Self {
            Self {
                answers: RefCell::new(answers.into()),
            }
        }

        fn pop(&self, expected: &str) -> MockAnswer {
            self.answers
                .borrow_mut()
                .pop_front()
                .unwrap_or_else(|| panic!("MockPrompter exhausted — expected {expected}"))
        }
    }

    impl Prompter for MockPrompter {
        fn prompt_text(&self, _message: &str) -> Result<String, Box<dyn Error>> {
            match self.pop("Text") {
                MockAnswer::Text(s) => Ok(s),
                _ => panic!("MockPrompter type mismatch: expected Text"),
            }
        }

        fn prompt_optional_text(&self, _message: &str) -> Result<Option<String>, Box<dyn Error>> {
            match self.pop("OptionalText") {
                MockAnswer::OptionalText(s) => Ok(s),
                _ => panic!("MockPrompter type mismatch: expected OptionalText"),
            }
        }

        fn prompt_select(
            &self,
            _message: &str,
            _options: &[&str],
        ) -> Result<String, Box<dyn Error>> {
            match self.pop("Select") {
                MockAnswer::Select(s) => Ok(s),
                _ => panic!("MockPrompter type mismatch: expected Select"),
            }
        }

        fn prompt_optional_select(
            &self,
            _message: &str,
            _options: &[&str],
        ) -> Result<Option<String>, Box<dyn Error>> {
            match self.pop("OptionalSelect") {
                MockAnswer::OptionalSelect(s) => Ok(s),
                _ => panic!("MockPrompter type mismatch: expected OptionalSelect"),
            }
        }

        fn prompt_optional_select_id(
            &self,
            _message: &str,
            _choices: &[super::IdChoice],
        ) -> Result<Option<i64>, Box<dyn Error>> {
            match self.pop("OptionalI64") {
                MockAnswer::OptionalI64(v) => Ok(v),
                _ => panic!("MockPrompter type mismatch: expected OptionalI64"),
            }
        }

        fn prompt_multi_select_id(
            &self,
            _message: &str,
            _choices: &[super::IdChoice],
        ) -> Result<Vec<i64>, Box<dyn Error>> {
            match self.pop("MultiSelectId") {
                MockAnswer::MultiSelectId(v) => Ok(v),
                _ => panic!("MockPrompter type mismatch: expected MultiSelectId"),
            }
        }

        fn prompt_optional_select_uuid(
            &self,
            _message: &str,
            _choices: &[super::UuidChoice],
        ) -> Result<Option<uuid::Uuid>, Box<dyn Error>> {
            match self.pop("OptionalUuid") {
                MockAnswer::OptionalUuid(v) => Ok(v),
                _ => panic!("MockPrompter type mismatch: expected OptionalUuid"),
            }
        }

        fn prompt_list(&self, _message: &str) -> Result<Vec<String>, Box<dyn Error>> {
            match self.pop("List") {
                MockAnswer::List(v) => Ok(v),
                _ => panic!("MockPrompter type mismatch: expected List"),
            }
        }

        fn prompt_multi_select(
            &self,
            _message: &str,
            _choices: &[super::MemberChoice],
        ) -> Result<Vec<String>, Box<dyn Error>> {
            match self.pop("MultiSelect") {
                MockAnswer::MultiSelect(v) => Ok(v),
                _ => panic!("MockPrompter type mismatch: expected MultiSelect"),
            }
        }

        fn prompt_optional_i64(&self, _message: &str) -> Result<Option<i64>, Box<dyn Error>> {
            match self.pop("OptionalI64") {
                MockAnswer::OptionalI64(v) => Ok(v),
                _ => panic!("MockPrompter type mismatch: expected OptionalI64"),
            }
        }

        fn confirm(&self, _message: &str) -> Result<bool, Box<dyn Error>> {
            match self.pop("Confirm") {
                MockAnswer::Confirm(b) => Ok(b),
                _ => panic!("MockPrompter type mismatch: expected Confirm"),
            }
        }
    }
}
