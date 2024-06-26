use core::ops::Mul;

use num::{Float, Zero};
use option_trait::{Maybe, StaticMaybe};

use crate::quantities::{IntoList, ListOrSingle};

pub trait PulseTrain<T, L, N>: IntoList<T, L, N>
where
    T: Float,
    L: ListOrSingle<T>,
    N: Maybe<usize>
{
    fn pulse_train<D, G, GG, P, R, O>(self, numtaps: N, train: D, pulse: P, fold: R) -> (D::Mapped<L::Mapped<O>>, L::Mapped<O>, L)
    where
        D: ListOrSingle<(T, GG)>,
        GG: StaticMaybe<G, MaybeOr<G, T> = G> + Clone,
        G: Clone,
        D::Mapped<L::Mapped<O>>: ListOrSingle<L::Mapped<O>>,
        L::Mapped<O>: ListOrSingle<O>,
        P: FnMut<(T,)>,
        R: Fn(O, O) -> O,
        O: Zero + Clone,
        P::Output: Clone + Mul<G, Output = O>;
}

impl<T, L, RR, N> PulseTrain<T, L, N> for RR
where
    T: Float,
    L: ListOrSingle<T>,
    RR: IntoList<T, L, N>,
    N: Maybe<usize>
{
    fn pulse_train<D, G, GG, P, R, O>(self, n: N, train: D, mut pulse: P, fold: R) -> (D::Mapped<L::Mapped<O>>, L::Mapped<O>, L)
    where
        D: ListOrSingle<(T, GG)>,
        GG: StaticMaybe<G, MaybeOr<G, T> = G> + Clone,
        G: Clone,
        D::Mapped<L::Mapped<O>>: ListOrSingle<L::Mapped<O>>,
        L::Mapped<O>: ListOrSingle<O>,
        P: FnMut<(T,)>,
        R: Fn(O, O) -> O,
        O: Zero + Clone,
        P::Output: Clone + Mul<G, Output = O>
    {
        let t = self.into_list(n);

        let s = train.map_into_owned(|(d, g)| {
            let g: G = g.into_option()
                .unwrap_or_else(|| unsafe {core::mem::transmute_copy(&T::one())});
            t.map_to_owned(|&t| {
                pulse(t - d)*g.clone()
            })
        });

        let ss = s.as_view_slice();

        let mut i = 0;
        let y = t.map_to_owned(|_| {
            let y = ss.iter()
                .map(|s| s.as_view_slice()[i].clone())
                .reduce(&fold)
                .unwrap_or_else(O::zero);
            i += 1;
            y
        });

        (s, y, t)
    }
}

#[cfg(test)]
mod test
{
    use core::ops::Add;

    use array_math::ArrayOps;

    use crate::{plot, gen::pulse::{GausPuls, PulseTrain}};

    #[test]
    fn test()
    {
        const N: usize = 1024;
        let (_s, y, t): (_, [_; N], _) = (0.0..=8.0).pulse_train((), [
            (1.0, 1.0),
            (3.0, 0.66),
            (6.0, 0.33)
        ], |x| {
            let ([y], [_], [_], [_]) = [x].gauspuls((), 5.0, 0.5);
            y
        }, Add::add);

        plot::plot_curves("y(t)", "plots/y_t_pulse_train.png", [&t.zip(y)])
            .unwrap()
    }
}