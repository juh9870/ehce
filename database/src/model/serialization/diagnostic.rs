use super::{DeserializationErrorKind, DeserializationErrorStackItem};
use crate::model::serialization::DeserializationError;
use miette::Diagnostic;
use std::fmt::{Display, Formatter};
use thiserror::Error;

#[derive(Debug)]
enum ItemDiagnosticKind {
    Path(DeserializationErrorStackItem),
    Cause(DeserializationErrorKind),
}

impl Display for ItemDiagnosticKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            ItemDiagnosticKind::Path(path) => match path {
                DeserializationErrorStackItem::Item(id, kind) => {
                    write!(f, "Failed to deserialize {kind}({id})")
                }
                DeserializationErrorStackItem::Field(field) => {
                    write!(f, "Failed to deserialize field `{field}`")
                }
                DeserializationErrorStackItem::Index(i) => {
                    write!(f, "Failed to deserialize array item at position `{i}`")
                }
                DeserializationErrorStackItem::MapEntry(key) => {
                    write!(f, "Failed to deserialize map entry with key `{key}`")
                }
                DeserializationErrorStackItem::ExprVariable(name) => {
                    write!(f, "Failed to resolve expression variable `{name}`")
                }
            },
            ItemDiagnosticKind::Cause(cause) => {
                write!(f, "{cause}")
            }
        }
    }
}

#[derive(Debug, Error)]
#[error("{}", .0)]
struct ItemDiagnostic(ItemDiagnosticKind, Option<Box<ItemDiagnostic>>);

impl Diagnostic for ItemDiagnostic {
    fn diagnostic_source(&self) -> Option<&dyn Diagnostic> {
        self.1.as_ref().map(|e| e.as_ref() as &dyn Diagnostic)
    }
}

impl DeserializationError {
    pub fn diagnostic(self) -> impl Diagnostic {
        self.stack.into_iter().fold(
            ItemDiagnostic(ItemDiagnosticKind::Cause(self.kind), None),
            |err, item| ItemDiagnostic(ItemDiagnosticKind::Path(item), Some(Box::new(err))),
        )
    }
}
