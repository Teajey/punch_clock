pub trait Span<T> {
    type Output;

    fn span(&self) -> Self::Output;
}
