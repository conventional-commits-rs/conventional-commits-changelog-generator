//! Contains utility functions.

pub(crate) fn pairwise<I, J>(right: I) -> impl Iterator<Item = (I::Item, I::Item)>
where
    I: IntoIterator<Item = J> + Clone,
    J: std::fmt::Display,
{
    let left = right.clone().into_iter();
    let right = right.into_iter().skip(1);
    left.zip(right)
}
