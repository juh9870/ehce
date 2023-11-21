use miette::Diagnostic;
use std::fmt::Display;
use thiserror::Error;

pub trait DiagnosticWrapper: sealed::Sealed {
    type Wrapped;
    fn wrap(self, message: impl Display) -> Self::Wrapped;
}

#[derive(Debug, Error)]
#[error("{}", .message)]
pub struct WrappedDiagnostic<T: Diagnostic> {
    pub message: String,
    pub cause: T,
}

impl<T: Diagnostic> Diagnostic for WrappedDiagnostic<T> {
    fn diagnostic_source(&self) -> Option<&dyn Diagnostic> {
        Some(&self.cause)
    }
}

fn context<T: Diagnostic>(diagnostic: T, message: impl Display) -> WrappedDiagnostic<T> {
    WrappedDiagnostic {
        message: message.to_string(),
        cause: diagnostic,
    }
}

impl<T: Diagnostic> DiagnosticWrapper for T {
    type Wrapped = WrappedDiagnostic<T>;

    fn wrap(self, message: impl Display) -> Self::Wrapped {
        context(self, message)
    }
}

mod sealed {
    use miette::Diagnostic;

    pub trait Sealed {}
    impl<T: Diagnostic> Sealed for T {}
}
