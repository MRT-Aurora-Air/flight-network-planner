#[inline]
#[expect(clippy::impl_trait_in_params)]
pub fn for_both<T, U: Sized>(x: &T, y: &T, f: impl Fn(&T) -> U) -> (U, U) {
    (f(x), f(y))
}

#[inline]
#[expect(clippy::impl_trait_in_params)]
pub fn for_both_permutations<T, U: Sized>(x: &T, y: &T, mut f: impl FnMut(&T, &T) -> U) -> (U, U) {
    (f(x, y), f(y, x))
}

#[macro_export]
macro_rules! fbp {
    (filter $f:expr) => {
        |(x, y)| {
            let (a, b) = $crate::utils::for_both_permutations(x, y, $f);
            a && b
        }
    };
}

#[expect(dead_code)]
pub trait AnyAllBool: Copy {
    fn any(self) -> bool;
    fn all(self) -> bool;
}
impl AnyAllBool for (bool, bool) {
    #[inline]
    fn any(self) -> bool {
        self.0 || self.1
    }
    fn all(self) -> bool {
        self.0 && self.1
    }
}
