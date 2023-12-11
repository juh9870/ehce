use miette::Diagnostic;
use std::fmt::Display;
use thiserror::Error;

pub use paste::paste;

#[macro_export]
macro_rules! _bubbled_impl {
    ($name:ident, $($variant:ty),*) => {
        $crate::miette_ext::paste! {
            #[derive(Debug, thiserror::Error, miette::Diagnostic)]
            pub enum $name {
                $(
                    [<$variant>](#[diagnostic_source] $variant)
                ),*
            }

            $(
                #[automatically_derived]
                impl From<$variant> for $name {
                    #[inline(always)]
                    fn from(value: $variant) -> Self {
                        Self::[<$variant>](value)
                    }
                }
            )*
        }
    };
}

#[macro_export]
macro_rules! bubbled {
    ($name:ident($message:literal) { $($variant:ty),* $(,)? }) => {
        $crate::_bubbled_impl!($name, $($variant),*);

        #[automatically_derived]
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", $message)
            }
        }
    };
    ($name:ident { $($variant:ty),* $(,)? }) => {
        $crate::_bubbled_impl!($name, $($variant),*);


        $crate::miette_ext::paste! {
            #[automatically_derived]
            impl std::fmt::Display for $name {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    match self {
                        $(
                            Self::[<$variant>](data) => write!(f, "{}", data)
                        ),*
                    }
                }
            }
        }
    };
}

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
