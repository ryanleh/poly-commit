use crate::{Polynomial, Rc, String, Vec};
use ark_ff::{Field, Zero};
use ark_std::{
    borrow::Borrow,
    marker::PhantomData,
    ops::{Add, AddAssign, MulAssign, SubAssign},
};
use crypto_primitives::Share;
use rand_core::RngCore;

/// Labels a `LabeledPolynomial` or a `LabeledCommitment`.
pub type PolynomialLabel = String;

/// Defines the minimal interface for public params for any polynomial
/// commitment scheme.
pub trait PCUniversalParams: Clone + core::fmt::Debug {
    /// Outputs the maximum degree supported by the committer key.
    fn max_degree(&self) -> usize;
}

/// Defines the minimal interface of committer keys for any polynomial
/// commitment scheme.
pub trait PCCommitterKey: Clone + core::fmt::Debug {
    /// Outputs the maximum degree supported by the universal parameters
    /// `Self` was derived from.
    fn max_degree(&self) -> usize;

    /// Outputs the maximum degree supported by the committer key.
    fn supported_degree(&self) -> usize;
}

/// Defines the minimal interface of verifier keys for any polynomial
/// commitment scheme.
pub trait PCVerifierKey: Clone + core::fmt::Debug {
    /// Outputs the maximum degree supported by the universal parameters
    /// `Self` was derived from.
    fn max_degree(&self) -> usize;

    /// Outputs the maximum degree supported by the verifier key.
    fn supported_degree(&self) -> usize;
}

/// Defines the minimal interface of prepared verifier keys for any polynomial
/// commitment scheme.
pub trait PCPreparedVerifierKey<Unprepared: PCVerifierKey> {
    /// prepare
    fn prepare(vk: &Unprepared) -> Self;
}

/// Defines the minimal interface of commitments for any polynomial
/// commitment scheme.
pub trait PCCommitment: Clone + ark_ff::ToBytes {
    /// Outputs a non-hiding commitment to the zero polynomial.
    fn empty() -> Self;

    /// Does this commitment have a degree bound?
    fn has_degree_bound(&self) -> bool;

    /// Size in bytes
    fn size_in_bytes(&self) -> usize;
}

/// Defines the minimal interface of prepared commitments for any polynomial
/// commitment scheme.
pub trait PCPreparedCommitment<UNPREPARED: PCCommitment>: Clone {
    /// prepare
    fn prepare(comm: &UNPREPARED) -> Self;
}

/// Defines the minimal interface of commitment randomness for any polynomial
/// commitment scheme.
pub trait PCRandomness: Clone {
    /// Outputs empty randomness that does not hide the commitment.
    fn empty() -> Self;

    /// Samples randomness for commitments;
    /// `num_queries` specifies the number of queries that the commitment will be opened at.
    /// `has_degree_bound` indicates that the corresponding commitment has an enforced
    /// `num_vars` specifies the number of variables for multivariate commitment.
    /// strict degree bound.
    fn rand<R: RngCore>(
        num_queries: usize,
        has_degree_bound: bool,
        num_vars: Option<usize>,
        rng: &mut R,
    ) -> Self;
}

/// Defines the minimal interface of evaluation proofs for any polynomial
/// commitment scheme.
pub trait PCProof: Clone + ark_ff::ToBytes {
    /// Size in bytes
    fn size_in_bytes(&self) -> usize;
}

/// A polynomial along with information about its degree bound (if any), and the
/// maximum number of queries that will be made to it. This latter number determines
/// the amount of protection that will be provided to a commitment for this polynomial.
#[derive(Debug, Clone)]
pub struct LabeledPolynomial<F: Field, P: Polynomial<F>> {
    label: PolynomialLabel,
    polynomial: Rc<P>,
    degree_bound: Option<usize>,
    hiding_bound: Option<usize>,
    _field: PhantomData<F>,
}

impl<'a, F: Field, P: Polynomial<F>> core::ops::Deref for LabeledPolynomial<F, P> {
    type Target = P;

    fn deref(&self) -> &Self::Target {
        &self.polynomial
    }
}

impl<'a, F: Field, P: Polynomial<F>> LabeledPolynomial<F, P> {
    /// Construct a new labeled polynomial.
    pub fn new(
        label: PolynomialLabel,
        polynomial: P,
        degree_bound: Option<usize>,
        hiding_bound: Option<usize>,
    ) -> Self {
        Self {
            label,
            polynomial: Rc::new(polynomial),
            degree_bound,
            hiding_bound,
            _field: PhantomData,
        }
    }

    /// Return the label for `self`.
    pub fn label(&self) -> &String {
        &self.label
    }

    /// Retrieve the polynomial from `self`
    pub fn polynomial(&self) -> &P {
        &self.polynomial
    }

    /// Evaluate the polynomial in `self`.
    pub fn evaluate(&self, point: &P::Point) -> F {
        self.polynomial.evaluate(point)
    }

    /// Retrieve the degree of the polynomial in `self`.
    pub fn degree(&self) -> usize {
        self.polynomial.degree()
    }

    /// Retrieve the degree bound in `self`.
    pub fn degree_bound(&self) -> Option<usize> {
        self.degree_bound
    }

    /// Retrieve whether the polynomial in `self` should be hidden.
    pub fn is_hiding(&self) -> bool {
        self.hiding_bound.is_some()
    }

    /// Retrieve the hiding bound for the polynomial in `self`.
    pub fn hiding_bound(&self) -> Option<usize> {
        self.hiding_bound
    }
}

impl<F: Field, P: Polynomial<F>> Add for LabeledPolynomial<F, P> {
    type Output = Self;

    #[inline]
    fn add(self, mut other: Self) -> Self {
        *Rc::get_mut(&mut other.polynomial).unwrap() += self.polynomial.as_ref();
        other
    }
}

impl<F: Field, P: Polynomial<F>> Zero for LabeledPolynomial<F, P> {
    #[inline]
    fn zero() -> Self {
        Self::new(String::new(), P::zero(), None, None)
    }

    #[inline]
    fn is_zero(&self) -> bool {
        unimplemented!()
    }
}

impl<F: Field, P: Polynomial<F> + Share> Share for LabeledPolynomial<F, P> {
    fn share<R: RngCore>(&self, num: usize, rng: &mut R) -> Vec<Self> {
        self.polynomial
            .share(num, rng)
            .into_iter()
            .map(|p| Self::new(self.label.clone(), p, self.degree_bound, self.hiding_bound))
            .collect()
    }
}


/// A commitment along with information about its degree bound (if any).
#[derive(Clone)]
pub struct LabeledCommitment<C: PCCommitment> {
    label: PolynomialLabel,
    commitment: C,
    degree_bound: Option<usize>,
}

impl<C: PCCommitment> LabeledCommitment<C> {
    /// Instantiate a new polynomial_context.
    pub fn new(label: PolynomialLabel, commitment: C, degree_bound: Option<usize>) -> Self {
        Self {
            label,
            commitment,
            degree_bound,
        }
    }

    /// Return the label for `self`.
    pub fn label(&self) -> &String {
        &self.label
    }

    /// Retrieve the commitment from `self`.
    pub fn commitment(&self) -> &C {
        &self.commitment
    }

    /// Retrieve the degree bound in `self`.
    pub fn degree_bound(&self) -> Option<usize> {
        self.degree_bound
    }
}

impl<C: PCCommitment> ark_ff::ToBytes for LabeledCommitment<C> {
    #[inline]
    fn write<W: ark_std::io::Write>(&self, writer: W) -> ark_std::io::Result<()> {
        self.commitment.write(writer)
    }
}

impl<C: PCCommitment + Add<Output=C>> Add for LabeledCommitment<C> {
    type Output = Self;

    #[inline]
    fn add(self, mut other: Self) -> Self {
        other.commitment = self.commitment + other.commitment;
        other
    }
}

impl<C: PCCommitment + Zero> Zero for LabeledCommitment<C> {
    #[inline]
    fn zero() -> Self {
        Self::new(String::new(), C::zero(), None)
    }

    #[inline]
    fn is_zero(&self) -> bool {
        unimplemented!()
    }
}

impl<C: PCCommitment + Share> Share for LabeledCommitment<C> {
    fn share<R: RngCore>(&self, num: usize, rng: &mut R) -> Vec<Self> {
        self.commitment
            .share(num, rng)
            .into_iter()
            .map(|c| Self::new(self.label.clone(), c, self.degree_bound))
            .collect()
    }
}

/// A term in a linear combination.
#[derive(Hash, Ord, PartialOrd, Clone, Eq, PartialEq, Debug)]
pub enum LCTerm {
    /// The constant term representing `one`.
    One,
    /// Label for a polynomial.
    PolyLabel(String),
}

impl LCTerm {
    /// Returns `true` if `self == LCTerm::One`
    #[inline]
    pub fn is_one(&self) -> bool {
        if let LCTerm::One = self {
            true
        } else {
            false
        }
    }
}

impl From<PolynomialLabel> for LCTerm {
    fn from(other: PolynomialLabel) -> Self {
        Self::PolyLabel(other)
    }
}

impl<'a> From<&'a str> for LCTerm {
    fn from(other: &str) -> Self {
        Self::PolyLabel(other.into())
    }
}

impl core::convert::TryInto<PolynomialLabel> for LCTerm {
    type Error = ();
    fn try_into(self) -> Result<PolynomialLabel, ()> {
        match self {
            Self::One => Err(()),
            Self::PolyLabel(l) => Ok(l),
        }
    }
}

impl<'a> core::convert::TryInto<&'a PolynomialLabel> for &'a LCTerm {
    type Error = ();

    fn try_into(self) -> Result<&'a PolynomialLabel, ()> {
        match self {
            LCTerm::One => Err(()),
            LCTerm::PolyLabel(l) => Ok(l),
        }
    }
}

impl<B: Borrow<String>> PartialEq<B> for LCTerm {
    fn eq(&self, other: &B) -> bool {
        match self {
            Self::One => false,
            Self::PolyLabel(l) => l == other.borrow(),
        }
    }
}

/// A labeled linear combinations of polynomials.
#[derive(Clone, Debug)]
pub struct LinearCombination<F> {
    /// The label.
    pub label: String,
    /// The linear combination of `(coeff, poly_label)` pairs.
    pub terms: Vec<(F, LCTerm)>,
}

impl<F: Field> LinearCombination<F> {
    /// Construct an empty labeled linear combination.
    pub fn empty(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            terms: Vec::new(),
        }
    }

    /// Construct a new labeled linear combination.
    /// with the terms specified in `term`.
    pub fn new(label: impl Into<String>, terms: Vec<(F, impl Into<LCTerm>)>) -> Self {
        let terms = terms.into_iter().map(|(c, t)| (c, t.into())).collect();
        Self {
            label: label.into(),
            terms: terms,
        }
    }

    /// Returns the label of the linear combination.
    pub fn label(&self) -> &String {
        &self.label
    }

    /// Returns `true` if the linear combination has no terms.
    pub fn is_empty(&self) -> bool {
        self.terms.is_empty()
    }

    /// Add a term to the linear combination.
    pub fn push(&mut self, term: (F, LCTerm)) -> &mut Self {
        self.terms.push(term);
        self
    }
}

impl<'a, F: Field> AddAssign<(F, &'a LinearCombination<F>)> for LinearCombination<F> {
    fn add_assign(&mut self, (coeff, other): (F, &'a LinearCombination<F>)) {
        self.terms
            .extend(other.terms.iter().map(|(c, t)| (coeff * c, t.clone())));
    }
}

impl<'a, F: Field> SubAssign<(F, &'a LinearCombination<F>)> for LinearCombination<F> {
    fn sub_assign(&mut self, (coeff, other): (F, &'a LinearCombination<F>)) {
        self.terms
            .extend(other.terms.iter().map(|(c, t)| (-coeff * c, t.clone())));
    }
}

impl<'a, F: Field> AddAssign<&'a LinearCombination<F>> for LinearCombination<F> {
    fn add_assign(&mut self, other: &'a LinearCombination<F>) {
        self.terms.extend(other.terms.iter().cloned());
    }
}

impl<'a, F: Field> SubAssign<&'a LinearCombination<F>> for LinearCombination<F> {
    fn sub_assign(&mut self, other: &'a LinearCombination<F>) {
        self.terms
            .extend(other.terms.iter().map(|(c, t)| (-*c, t.clone())));
    }
}

impl<F: Field> AddAssign<F> for LinearCombination<F> {
    fn add_assign(&mut self, coeff: F) {
        self.terms.push((coeff, LCTerm::One));
    }
}

impl<F: Field> SubAssign<F> for LinearCombination<F> {
    fn sub_assign(&mut self, coeff: F) {
        self.terms.push((-coeff, LCTerm::One));
    }
}

impl<F: Field> MulAssign<F> for LinearCombination<F> {
    fn mul_assign(&mut self, coeff: F) {
        self.terms.iter_mut().for_each(|(c, _)| *c *= coeff);
    }
}

impl<F: Field> core::ops::Deref for LinearCombination<F> {
    type Target = [(F, LCTerm)];

    fn deref(&self) -> &Self::Target {
        &self.terms
    }
}
