use heck::{
    ToKebabCase, ToLowerCamelCase, ToShoutyKebabCase, ToShoutySnakeCase, ToSnakeCase, ToTrainCase,
    ToUpperCamelCase,
};

#[derive(Clone, Copy)]
pub(crate) enum RenameRule {
    LowerCase,
    UpperCase,
    PascalCase,
    CamelCase,
    SnakeCase,
    ScreamingSnakeCase,
    KebabCase,
    ScreamingKebabCase,
    TrainCase,
}

impl RenameRule {
    pub(crate) fn from_str(rule: &str) -> Option<Self> {
        match rule {
            "lowercase" => Some(Self::LowerCase),
            "UPPERCASE" => Some(Self::UpperCase),
            "PascalCase" => Some(Self::PascalCase),
            "camelCase" => Some(Self::CamelCase),
            "snake_case" => Some(Self::SnakeCase),
            "SCREAMING_SNAKE_CASE" => Some(Self::ScreamingSnakeCase),
            "kebab-case" => Some(Self::KebabCase),
            "SCREAMING-KEBAB-CASE" => Some(Self::ScreamingKebabCase),
            "Train-Case" => Some(Self::TrainCase),
            _ => None,
        }
    }

    pub(crate) fn apply_to_field(self, field: &str) -> String {
        match self {
            Self::LowerCase => field.to_lowercase(),
            Self::UpperCase => field.to_uppercase(),
            Self::PascalCase => field.to_upper_camel_case(),
            Self::CamelCase => field.to_lower_camel_case(),
            Self::SnakeCase => field.to_snake_case(),
            Self::ScreamingSnakeCase => field.to_shouty_snake_case(),
            Self::KebabCase => field.to_kebab_case(),
            Self::ScreamingKebabCase => field.to_shouty_kebab_case(),
            Self::TrainCase => field.to_train_case(),
        }
    }
}
