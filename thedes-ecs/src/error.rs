use std::{collections::HashMap, fmt};

pub type CtxResult<T, E> = Result<T, ErrorContext<E>>;

pub trait OptionExt {
    type Item;

    fn ok_or_ctx<E>(self, error: E) -> CtxResult<Self::Item, E>
    where
        E: fmt::Display;

    fn ok_or_else_ctx<F, E>(self, f: F) -> CtxResult<Self::Item, E>
    where
        F: FnOnce() -> E,
        E: fmt::Display;
}

pub trait ResultWrapExt {
    type WrapOk;
    type WrapErr: fmt::Display;

    fn wrap_ctx(self) -> CtxResult<Self::WrapOk, Self::WrapErr>;
}

pub trait ResultMapExt: Sized {
    type MapOk;
    type MapErr: fmt::Display;

    fn map_cause<F, E0>(self, f: F) -> CtxResult<Self::MapOk, E0>
    where
        F: FnOnce(Self::MapErr) -> E0,
        E0: fmt::Display;

    fn cause_into<E0>(self) -> CtxResult<Self::MapOk, E0>
    where
        Self::MapErr: Into<E0>,
        E0: fmt::Display;

    fn map_info<F>(self, f: F) -> CtxResult<Self::MapOk, Self::MapErr>
    where
        F: FnOnce(ErrorInfo) -> ErrorInfo;

    fn adding_info(
        self,
        key: &str,
        value: impl fmt::Display,
    ) -> CtxResult<Self::MapOk, Self::MapErr> {
        self.map_info(|info| info.adding(key, value))
    }

    fn renaming_info(
        self,
        key: &str,
        new_key: &str,
    ) -> CtxResult<Self::MapOk, Self::MapErr> {
        self.map_info(|info| info.renaming(key, new_key))
    }

    fn removing_info(self, key: &str) -> CtxResult<Self::MapOk, Self::MapErr> {
        self.map_info(|info| info.removing(key))
    }
}

impl<T> OptionExt for Option<T> {
    type Item = T;

    fn ok_or_ctx<E>(self, error: E) -> CtxResult<Self::Item, E>
    where
        E: fmt::Display,
    {
        self.ok_or(error).wrap_ctx()
    }

    fn ok_or_else_ctx<F, E>(self, f: F) -> CtxResult<Self::Item, E>
    where
        F: FnOnce() -> E,
        E: fmt::Display,
    {
        self.ok_or_else(f).wrap_ctx()
    }
}

impl<T, E> ResultWrapExt for Result<T, E>
where
    E: fmt::Display,
{
    type WrapOk = T;
    type WrapErr = E;

    fn wrap_ctx(self) -> CtxResult<Self::WrapOk, Self::WrapErr> {
        self.map_err(ErrorContext::new)
    }
}

impl<T, E> ResultMapExt for Result<T, ErrorContext<E>>
where
    E: fmt::Display,
{
    type MapOk = T;
    type MapErr = E;

    fn map_cause<F, E0>(self, f: F) -> Result<Self::MapOk, ErrorContext<E0>>
    where
        F: FnOnce(Self::MapErr) -> E0,
        E0: fmt::Display,
    {
        self.map_err(|e| e.map_cause(f))
    }

    fn cause_into<E0>(self) -> CtxResult<Self::MapOk, E0>
    where
        Self::MapErr: Into<E0>,
        E0: fmt::Display,
    {
        self.map_err(|e| e.cause_into::<E0>())
    }

    fn map_info<F>(self, f: F) -> CtxResult<Self::MapOk, Self::MapErr>
    where
        F: FnOnce(ErrorInfo) -> ErrorInfo,
    {
        self.map_err(|e| e.map_info(f))
    }
}

#[derive(Debug)]
pub struct ErrorContext<E> {
    inner: Box<Inner<E>>,
}

impl<E> ErrorContext<E>
where
    E: fmt::Display,
{
    pub fn new(cause: E) -> Self {
        Self {
            inner: Box::new(Inner {
                info: ErrorInfo::default(),
                cause: Box::new(cause),
            }),
        }
    }

    pub fn info(&self) -> &ErrorInfo {
        &self.inner.info
    }

    pub fn info_mut(&mut self) -> &mut ErrorInfo {
        &mut self.inner.info
    }

    pub fn cause(&self) -> &E {
        &self.inner.cause
    }

    pub fn into_cause(self) -> E {
        *self.inner.cause
    }

    pub fn map_cause<F, E0>(self, f: F) -> ErrorContext<E0>
    where
        F: FnOnce(E) -> E0,
        E0: fmt::Display,
    {
        let Inner { info, cause } = *self.inner;
        ErrorContext {
            inner: Box::new(Inner { info, cause: Box::new(f(*cause)) }),
        }
    }

    pub fn cause_into<E0>(self) -> ErrorContext<E0>
    where
        E: Into<E0>,
        E0: fmt::Display,
    {
        self.map_cause(Into::into)
    }

    pub fn map_info<F>(self, f: F) -> Self
    where
        F: FnOnce(ErrorInfo) -> ErrorInfo,
    {
        let Inner { info, cause } = *self.inner;
        ErrorContext { inner: Box::new(Inner { info: f(info), cause }) }
    }
}

impl<E> fmt::Display for ErrorContext<E>
where
    E: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.cause(), self.info())
    }
}

impl<E> std::error::Error for ErrorContext<E>
where
    E: std::error::Error + 'static,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.cause())
    }
}

#[derive(Debug)]
struct Inner<E> {
    info: ErrorInfo,
    cause: Box<E>,
}

#[derive(Debug, Clone, PartialEq, Default)]
#[non_exhaustive]
pub struct ErrorInfo {
    attrs: HashMap<String, Vec<String>>,
}

impl ErrorInfo {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(
        &self,
        key: &str,
    ) -> impl Iterator<Item = &str> + Send + fmt::Debug {
        self.attrs.get(key).into_iter().flatten().map(String::as_str)
    }

    pub fn iter(
        &self,
    ) -> impl Iterator<Item = (&str, &str)> + Send + fmt::Debug {
        self.attrs.iter().flat_map(|(key, entries)| {
            entries.iter().map(|entry| (key.as_str(), entry.as_str()))
        })
    }

    pub fn add(&mut self, key: &str, value: impl fmt::Display) -> &mut Self {
        match self.attrs.get_mut(key) {
            Some(entry) => {
                entry.push(value.to_string());
            },
            None => {
                self.attrs.insert(key.to_owned(), vec![value.to_string()]);
            },
        }
        self
    }

    pub fn adding(mut self, key: &str, value: impl fmt::Display) -> Self {
        self.add(key, value);
        self
    }

    pub fn rename(&mut self, key: &str, new_key: &str) -> &mut Self {
        if let Some(mut old) = self.attrs.remove(key) {
            match self.attrs.get_mut(new_key) {
                Some(entry) => {
                    entry.append(&mut old);
                },
                None => {
                    self.attrs.insert(new_key.to_owned(), old);
                },
            }
        }
        self
    }

    pub fn renaming(mut self, key: &str, new_key: &str) -> Self {
        self.rename(key, new_key);
        self
    }

    pub fn remove(
        &mut self,
        key: &str,
    ) -> impl Iterator<Item = String> + Send + fmt::Debug {
        self.attrs.remove(key).into_iter().flatten()
    }

    pub fn removing(mut self, key: &str) -> Self {
        self.remove(key).for_each(drop);
        self
    }
}

impl fmt::Display for ErrorInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[ ")?;
        write!(f, "]")?;
        Ok(())
    }
}
