pub(crate) struct BiasedAlternator<T, I1, I2> {
    next: Next,
    first: I1,
    second: I2,
    condition: Option<SwitchCondition<T>>,
}

type SwitchCondition<T> = Box<dyn Fn(&T) -> bool + Send>;

enum Next {
    First,
    Second,
}

impl<T, I1, I2> BiasedAlternator<T, I1, I2> {
    pub(crate) fn on(mut self, condition: impl Fn(&T) -> bool + Send + 'static) -> Self {
        self.condition = Some(Box::new(condition));
        self
    }
}

impl<T, I1, I2> Iterator for BiasedAlternator<T, I1, I2>
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
            condition,
        } = self;
        match next {
            Next::First => {
                let ret = first.next();
                let switch = ret.as_ref().is_some_and(|elem| {
                    condition
                        .as_ref()
                        .map(|condition| condition(elem))
                        .unwrap_or(true)
                });
                if switch {
                    *next = Next::Second;
                }
                ret
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

    fn biased_alternate_with<I>(self, iter: I) -> BiasedAlternator<Self::Item, Self, I::IntoIter>
    where
        I: IntoIterator<Item = Self::Item> + Sized;
}

impl<T, I> BiasedAlternateWithExt for I
where
    I: Iterator<Item = T>,
{
    type Item = T;

    fn biased_alternate_with<I2>(self, iter: I2) -> BiasedAlternator<T, Self, I2::IntoIter>
    where
        I2: IntoIterator<Item = Self::Item> + Sized,
    {
        let second = iter.into_iter();
        BiasedAlternator {
            next: Next::First,
            first: self,
            second,
            condition: None,
        }
    }
}
