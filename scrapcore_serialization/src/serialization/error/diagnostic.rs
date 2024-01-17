use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

use crate::registry::SerializationRegistry;
use miette::Diagnostic;

use super::{DeserializationError, DeserializationErrorKind, DeserializationErrorStackItem};

#[derive(Debug)]
enum ItemDiagnosticKind<Registry: SerializationRegistry> {
    Path(DeserializationErrorStackItem<Registry>),
    Cause(DeserializationErrorKind<Registry>),
}

impl<Registry: SerializationRegistry> Display for ItemDiagnosticKind<Registry> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            ItemDiagnosticKind::Path(path) => match path {
                DeserializationErrorStackItem::ItemByPath(path, kind) => {
                    write!(f, "Failed to load {kind} at {}", path.to_string_lossy())
                }
                DeserializationErrorStackItem::ItemById(id, kind) => {
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
                DeserializationErrorStackItem::MapKey(key) => {
                    write!(f, "Failed to deserialize map key `{key}`")
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

struct ItemDiagnostic<Registry: SerializationRegistry>(
    ItemDiagnosticKind<Registry>,
    Option<Box<ItemDiagnostic<Registry>>>,
);

impl<Registry: SerializationRegistry + Debug> Debug for ItemDiagnostic<Registry> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ItemDiagnostic")
            .field(&self.0)
            .field(&self.1)
            .finish()
    }
}

impl<Registry: SerializationRegistry> Display for ItemDiagnostic<Registry> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl<Registry: SerializationRegistry + Debug> Error for ItemDiagnostic<Registry> {}

impl<Registry: SerializationRegistry + Debug> Diagnostic for ItemDiagnostic<Registry> {
    fn diagnostic_source(&self) -> Option<&dyn Diagnostic> {
        self.1.as_ref().map(|e| e.as_ref() as &dyn Diagnostic)
    }
}

impl<Registry: SerializationRegistry> DeserializationError<Registry> {
    pub fn diagnostic(self) -> impl Diagnostic {
        self.stack.into_iter().fold(
            ItemDiagnostic(ItemDiagnosticKind::Cause(self.kind), None),
            |err, item| ItemDiagnostic(ItemDiagnosticKind::Path(item), Some(Box::new(err))),
        )
    }
}
