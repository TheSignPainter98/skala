pub(crate) struct BiasedAlternator<I1, I2> {
    next: Next,
    first: I1,
    second: I2,
}

enum Next {
    First,
    Second,
}

impl<T, I1, I2> Iterator for BiasedAlternator<I1, I2>
where
    I1: Iterator<Item = T>,
    I2: Iterator<Item = T>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let Self {
            next,
            first,
            second,
        } = self;
        match next {
            Next::First => {
                *next = Next::Second;
                first.next()
            }
            Next::Second => {
                *next = Next::First;
                second.next().or_else(|| first.next())
            }
        }
    }
}

pub(crate) trait BiasedAlternateWithExt: Sized {
    type Item;

    fn biased_alternate_with<I>(self, iter: I) -> BiasedAlternator<Self, I::IntoIter>
    where
        I: IntoIterator<Item = Self::Item> + Sized;
}

impl<T, I> BiasedAlternateWithExt for I
where
    I: Iterator<Item = T>,
{
    type Item = T;

    fn biased_alternate_with<I2>(self, iter: I2) -> BiasedAlternator<Self, I2::IntoIter>
    where
        I2: IntoIterator<Item = Self::Item> + Sized,
    {
        let second = iter.into_iter();
        BiasedAlternator {
            next: Next::First,
            first: self,
            second,
        }
    }
}
