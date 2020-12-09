use crate::{
    PCCommitment, PCCommitterKey, PCPreparedCommitment, PCPreparedVerifierKey, PCRandomness,
    PCVerifierKey, UVPolynomial, Vec,
};
use ark_ec::{PairingEngine, ProjectiveCurve};
use ark_ff::{PrimeField, ToBytes, Zero};
use ark_serialize::*;
use ark_std::ops::{Add, AddAssign};
use crypto_primitives::Share;
use rand_core::RngCore;

use crate::kzg10;
/// `UniversalParams` are the universal parameters for the KZG10 scheme.
pub type UniversalParams<E> = kzg10::UniversalParams<E>;

/// `CommitterKey` is used to commit to and create evaluation proofs for a given
/// polynomial.
#[derive(Derivative)]
#[derivative(
    Default(bound = ""),
    Hash(bound = ""),
    Clone(bound = ""),
    Debug(bound = "")
)]
pub struct CommitterKey<E: PairingEngine> {
    /// The key used to commit to polynomials.
    pub powers: Vec<E::G1Affine>,

    /// The key used to commit to shifted polynomials.
    /// This is `None` if `self` does not support enforcing any degree bounds.
    pub shifted_powers: Option<Vec<E::G1Affine>>,

    /// The key used to commit to hiding polynomials.
    pub powers_of_gamma_g: Vec<E::G1Affine>,

    /// The degree bounds that are supported by `self`.
    /// In ascending order from smallest to largest.
    /// This is `None` if `self` does not support enforcing any degree bounds.
    pub enforced_degree_bounds: Option<Vec<usize>>,
    /// The maximum degree supported by the `UniversalParams` `self` was derived
    /// from.
    pub max_degree: usize,
}

impl<E: PairingEngine> CanonicalSerialize for CommitterKey<E> {
    #[inline]
    fn serialize<W: Write>(&self, mut writer: W) -> Result<(), SerializationError> {
        self.powers.serialize_uncompressed(&mut writer)?;
        self.shifted_powers.serialize(&mut writer).unwrap();
        self.powers_of_gamma_g.serialize_uncompressed(&mut writer)?;
        self.enforced_degree_bounds.serialize(&mut writer)?;
        self.max_degree.serialize(&mut writer)?;
        Ok(())
    }

    #[inline]
    fn serialized_size(&self) -> usize {
        self.powers.uncompressed_size() +
        self.shifted_powers.uncompressed_size() +
        self.powers_of_gamma_g.uncompressed_size() +
        self.enforced_degree_bounds.serialized_size() +
        self.max_degree.serialized_size()
    }

    #[inline]
    fn serialize_uncompressed<W: Write>(&self, mut writer: W) -> Result<(), SerializationError> {
        self.powers.serialize_uncompressed(&mut writer)?;
        self.shifted_powers.serialize_uncompressed(&mut writer)?;
        self.powers_of_gamma_g.serialize_uncompressed(&mut writer)?;
        self.enforced_degree_bounds.serialize_uncompressed(&mut writer)?;
        self.max_degree.serialize_uncompressed(&mut writer)?;
        Ok(())
    }

    #[inline]
    fn uncompressed_size(&self) -> usize {
        self.powers.uncompressed_size() +
        self.shifted_powers.uncompressed_size() +
        self.powers_of_gamma_g.uncompressed_size() +
        self.enforced_degree_bounds.uncompressed_size() +
        self.max_degree.uncompressed_size()
    }
}

impl<E: PairingEngine> CanonicalDeserialize for CommitterKey<E> {
    #[inline]
    fn deserialize<R: Read>(mut reader: R) -> Result<Self, SerializationError> {
        let powers = Vec::<E::G1Affine>::deserialize_unchecked(&mut reader)?;
        let shifted_powers = Option::<Vec<E::G1Affine>>::deserialize(&mut reader)?;
        let powers_of_gamma_g = Vec::<E::G1Affine>::deserialize_unchecked(&mut reader)?;
        let enforced_degree_bounds = Option::<Vec<usize>>::deserialize(&mut reader)?;
        let max_degree = usize::deserialize(&mut reader)?;
        Ok(Self { powers, shifted_powers, powers_of_gamma_g, enforced_degree_bounds, max_degree })
    }

    #[inline]
    fn deserialize_uncompressed<R: Read>(mut reader: R) -> Result<Self, SerializationError> {
        let powers = Vec::<E::G1Affine>::deserialize_uncompressed(&mut reader)?;
        let shifted_powers = Option::<Vec<E::G1Affine>>::deserialize_uncompressed(&mut reader)?;
        let powers_of_gamma_g = Vec::<E::G1Affine>::deserialize_uncompressed(&mut reader)?;
        let enforced_degree_bounds = Option::<Vec<usize>>::deserialize_uncompressed(&mut reader)?;
        let max_degree = usize::deserialize_uncompressed(&mut reader)?;
        Ok(Self { powers, shifted_powers, powers_of_gamma_g, enforced_degree_bounds, max_degree })
    }

    #[inline]
    fn deserialize_unchecked<R: Read>(mut reader: R) -> Result<Self, SerializationError> {
        let powers = Vec::<E::G1Affine>::deserialize_unchecked(&mut reader)?;
        let shifted_powers = Option::<Vec<E::G1Affine>>::deserialize_unchecked(&mut reader)?;
        let powers_of_gamma_g = Vec::<E::G1Affine>::deserialize_unchecked(&mut reader)?;
        let enforced_degree_bounds = Option::<Vec<usize>>::deserialize_unchecked(&mut reader)?;
        let max_degree = usize::deserialize_unchecked(&mut reader)?;
        Ok(Self { powers, shifted_powers, powers_of_gamma_g, enforced_degree_bounds, max_degree })
    }
}



impl<E: PairingEngine> CommitterKey<E> {
    /// Obtain powers for the underlying KZG10 construction
    pub fn powers<'a>(&'a self) -> kzg10::Powers<'a, E> {
        kzg10::Powers {
            powers_of_g: self.powers.as_slice().into(),
            powers_of_gamma_g: self.powers_of_gamma_g.as_slice().into(),
        }
    }

    /// Obtain powers for committing to shifted polynomials.
    pub fn shifted_powers<'a>(
        &'a self,
        degree_bound: impl Into<Option<usize>>,
    ) -> Option<kzg10::Powers<'a, E>> {
        self.shifted_powers.as_ref().map(|shifted_powers| {
            let powers_range = if let Some(degree_bound) = degree_bound.into() {
                assert!(self
                    .enforced_degree_bounds
                    .as_ref()
                    .unwrap()
                    .contains(&degree_bound));
                let max_bound = self
                    .enforced_degree_bounds
                    .as_ref()
                    .unwrap()
                    .last()
                    .unwrap();
                (max_bound - degree_bound)..
            } else {
                0..
            };
            let ck = kzg10::Powers {
                powers_of_g: (&shifted_powers[powers_range]).into(),
                powers_of_gamma_g: self.powers_of_gamma_g.as_slice().into(),
            };
            ck
        })
    }
}

impl<E: PairingEngine> PCCommitterKey for CommitterKey<E> {
    fn max_degree(&self) -> usize {
        self.max_degree
    }

    fn supported_degree(&self) -> usize {
        self.powers.len()
    }
}

/// `VerifierKey` is used to check evaluation proofs for a given commitment.
#[derive(Derivative)]
#[derivative(Default(bound = ""), Clone(bound = ""), Debug(bound = ""))]
pub struct VerifierKey<E: PairingEngine> {
    /// The verification key for the underlying KZG10 scheme.
    pub vk: kzg10::VerifierKey<E>,
    /// Information required to enforce degree bounds. Each pair
    /// is of the form `(degree_bound, shifting_advice)`.
    /// The vector is sorted in ascending order of `degree_bound`.
    /// This is `None` if `self` does not support enforcing any degree bounds.
    pub degree_bounds_and_shift_powers: Option<Vec<(usize, E::G1Affine)>>,
    /// The maximum degree supported by the `UniversalParams` `self` was derived
    /// from.
    pub max_degree: usize,
    /// The maximum degree supported by the trimmed parameters that `self` is
    /// a part of.
    pub supported_degree: usize,
}

impl<E: PairingEngine> VerifierKey<E> {
    /// Find the appropriate shift for the degree bound.
    pub fn get_shift_power(&self, bound: usize) -> Option<E::G1Affine> {
        self.degree_bounds_and_shift_powers.as_ref().and_then(|v| {
            v.binary_search_by(|(d, _)| d.cmp(&bound))
                .ok()
                .map(|i| v[i].1)
        })
    }
}

impl<E: PairingEngine> PCVerifierKey for VerifierKey<E> {
    fn max_degree(&self) -> usize {
        self.max_degree
    }

    fn supported_degree(&self) -> usize {
        self.supported_degree
    }
}

impl<E: PairingEngine> ToBytes for VerifierKey<E> {
    #[inline]
    fn write<W: Write>(&self, mut writer: W) -> ark_std::io::Result<()> {
        self.vk.write(&mut writer)?;
        if let Some(degree_bounds_and_shift_powers) = &self.degree_bounds_and_shift_powers {
            writer.write_all(&degree_bounds_and_shift_powers.len().to_le_bytes())?;
            for (degree_bound, shift_power) in degree_bounds_and_shift_powers {
                writer.write_all(&degree_bound.to_le_bytes())?;
                shift_power.write(&mut writer)?;
            }
        }
        writer.write_all(&self.supported_degree.to_le_bytes())?;
        writer.write_all(&self.max_degree.to_le_bytes())
    }
}

/// `PreparedVerifierKey` is used to check evaluation proofs for a given commitment.
#[derive(Derivative)]
#[derivative(Clone(bound = ""), Debug(bound = ""))]
pub struct PreparedVerifierKey<E: PairingEngine> {
    /// The verification key for the underlying KZG10 scheme.
    pub prepared_vk: kzg10::PreparedVerifierKey<E>,
    /// Information required to enforce degree bounds. Each pair
    /// is of the form `(degree_bound, shifting_advice)`.
    /// This is `None` if `self` does not support enforcing any degree bounds.
    pub prepared_degree_bounds_and_shift_powers: Option<Vec<(usize, Vec<E::G1Affine>)>>,
    /// The maximum degree supported by the `UniversalParams` `self` was derived
    /// from.
    pub max_degree: usize,
    /// The maximum degree supported by the trimmed parameters that `self` is
    /// a part of.
    pub supported_degree: usize,
}

impl<E: PairingEngine> PCPreparedVerifierKey<VerifierKey<E>> for PreparedVerifierKey<E> {
    /// prepare `PreparedVerifierKey` from `VerifierKey`
    fn prepare(vk: &VerifierKey<E>) -> Self {
        let prepared_vk = kzg10::PreparedVerifierKey::<E>::prepare(&vk.vk);

        let supported_bits = E::Fr::size_in_bits();

        let prepared_degree_bounds_and_shift_powers: Option<Vec<(usize, Vec<E::G1Affine>)>> =
            if vk.degree_bounds_and_shift_powers.is_some() {
                let mut res = Vec::<(usize, Vec<E::G1Affine>)>::new();

                let degree_bounds_and_shift_powers =
                    vk.degree_bounds_and_shift_powers.as_ref().unwrap();

                for (d, shift_power) in degree_bounds_and_shift_powers {
                    let mut prepared_shift_power = Vec::<E::G1Affine>::new();

                    let mut cur = E::G1Projective::from(shift_power.clone());
                    for _ in 0..supported_bits {
                        prepared_shift_power.push(cur.clone().into());
                        cur.double_in_place();
                    }

                    res.push((d.clone(), prepared_shift_power));
                }

                Some(res)
            } else {
                None
            };

        Self {
            prepared_vk,
            prepared_degree_bounds_and_shift_powers,
            max_degree: vk.max_degree,
            supported_degree: vk.supported_degree,
        }
    }
}

/// Commitment to a polynomial that optionally enforces a degree bound.
#[derive(Derivative, CanonicalSerialize, CanonicalDeserialize)]
#[derivative(
    Default(bound = ""),
    Hash(bound = ""),
    Clone(bound = ""),
    Copy(bound = ""),
    Debug(bound = ""),
    PartialEq(bound = ""),
    Eq(bound = "")
)]
pub struct Commitment<E: PairingEngine> {
    /// A KZG10 commitment to the polynomial.
    pub comm: kzg10::Commitment<E>,

    /// A KZG10 commitment to the shifted polynomial.
    /// This is `none` if the committed polynomial does not
    /// enforce a strict degree bound.
    pub shifted_comm: Option<kzg10::Commitment<E>>,
}

impl<E: PairingEngine> ToBytes for Commitment<E> {
    #[inline]
    fn write<W: Write>(&self, mut writer: W) -> ark_std::io::Result<()> {
        self.comm.write(&mut writer)?;
        let shifted_exists = self.shifted_comm.is_some();
        shifted_exists.write(&mut writer)?;
        self.shifted_comm
            .as_ref()
            .unwrap_or(&kzg10::Commitment::empty())
            .write(&mut writer)
    }
}

impl<E: PairingEngine> PCCommitment for Commitment<E> {
    #[inline]
    fn empty() -> Self {
        Self {
            comm: kzg10::Commitment::empty(),
            shifted_comm: Some(kzg10::Commitment::empty()),
        }
    }

    fn has_degree_bound(&self) -> bool {
        self.shifted_comm.is_some()
    }

    fn size_in_bytes(&self) -> usize {
        self.comm.size_in_bytes() + self.shifted_comm.as_ref().map_or(0, |c| c.size_in_bytes())
    }
}

impl<E: PairingEngine> Add for Commitment<E> {
    type Output = Self;

    #[inline]
    fn add(self, other: Self) -> Self {
        let shifted_comm = match self.shifted_comm {
            Some(comm) => Some(comm + other.shifted_comm.unwrap()),
            None => other.shifted_comm,
        };
        Self {
            comm: self.comm + other.comm,
            shifted_comm,
        }
    }
}

impl<E: PairingEngine> Zero for Commitment<E> {
    #[inline]
    fn zero() -> Self {
        Self { 
            comm: kzg10::Commitment::zero(),
            shifted_comm: None,
        }
    }

    #[inline]
    fn is_zero(&self) -> bool {
        unimplemented!()
    }
}

impl<E: PairingEngine> Share for Commitment<E> {
    fn share<R: RngCore>(&self, num: usize, rng: &mut R) -> Vec<Self> {
        let comm_shares = self.comm.share(num, rng);
        if let Some(shifted) = self.shifted_comm {
            comm_shares
                .into_iter()
                .zip(shifted.share(num, rng))
                .map(|(c, s)| Self { comm: c, shifted_comm: Some(s) })
                .collect()
        } else {
            comm_shares
                .into_iter()
                .map(|comm| Self { comm, shifted_comm: None })
                .collect()
        }
    }
}

/// Prepared commitment to a polynomial that optionally enforces a degree bound.
#[derive(Derivative)]
#[derivative(
    Hash(bound = ""),
    Clone(bound = ""),
    Debug(bound = ""),
    PartialEq(bound = ""),
    Eq(bound = "")
)]
pub struct PreparedCommitment<E: PairingEngine> {
    pub(crate) prepared_comm: kzg10::PreparedCommitment<E>,
    pub(crate) shifted_comm: Option<kzg10::Commitment<E>>,
}

impl<E: PairingEngine> PCPreparedCommitment<Commitment<E>> for PreparedCommitment<E> {
    /// Prepare commitment to a polynomial that optionally enforces a degree bound.
    fn prepare(comm: &Commitment<E>) -> Self {
        let prepared_comm = kzg10::PreparedCommitment::<E>::prepare(&comm.comm);

        let shifted_comm = comm.shifted_comm.clone();

        Self {
            prepared_comm,
            shifted_comm,
        }
    }
}

/// `Randomness` hides the polynomial inside a commitment. It is output by `KZG10::commit`.
#[derive(Derivative)]
#[derivative(
    Hash(bound = ""),
    Clone(bound = ""),
    Debug(bound = ""),
    PartialEq(bound = ""),
    Eq(bound = "")
)]
pub struct Randomness<F: PrimeField, P: UVPolynomial<F>> {
    /// Commitment randomness for a KZG10 commitment.
    pub rand: kzg10::Randomness<F, P>,
    /// Commitment randomness for a KZG10 commitment to the shifted polynomial.
    /// This is `None` if the committed polynomial does not enforce a strict
    /// degree bound.
    pub shifted_rand: Option<kzg10::Randomness<F, P>>,
}

impl<'a, F: PrimeField, P: UVPolynomial<F>> Add<&'a Self> for Randomness<F, P> {
    type Output = Self;

    fn add(mut self, other: &'a Self) -> Self {
        self += other;
        self
    }
}

impl<'a, F: PrimeField, P: UVPolynomial<F>> AddAssign<&'a Self> for Randomness<F, P> {
    #[inline]
    fn add_assign(&mut self, other: &'a Self) {
        self.rand += &other.rand;
        if let Some(r1) = &mut self.shifted_rand {
            *r1 += other
                .shifted_rand
                .as_ref()
                .unwrap_or(&kzg10::Randomness::empty());
        } else {
            self.shifted_rand = other.shifted_rand.as_ref().map(|r| r.clone());
        }
    }
}

impl<'a, F: PrimeField, P: UVPolynomial<F>> Add<(F, &'a Randomness<F, P>)> for Randomness<F, P> {
    type Output = Self;

    #[inline]
    fn add(mut self, other: (F, &'a Randomness<F, P>)) -> Self {
        self += other;
        self
    }
}

impl<'a, F: PrimeField, P: UVPolynomial<F>> AddAssign<(F, &'a Randomness<F, P>)>
    for Randomness<F, P>
{
    #[inline]
    fn add_assign(&mut self, (f, other): (F, &'a Randomness<F, P>)) {
        self.rand += (f, &other.rand);
        let empty = kzg10::Randomness::empty();
        if let Some(r1) = &mut self.shifted_rand {
            *r1 += (f, other.shifted_rand.as_ref().unwrap_or(&empty));
        } else {
            self.shifted_rand = other.shifted_rand.as_ref().map(|r| empty + (f, r));
        }
    }
}

impl<F: PrimeField, P: UVPolynomial<F>> PCRandomness for Randomness<F, P> {
    fn empty() -> Self {
        Self {
            rand: kzg10::Randomness::empty(),
            shifted_rand: None,
        }
    }

    fn rand<R: RngCore>(
        hiding_bound: usize,
        has_degree_bound: bool,
        _: Option<usize>,
        rng: &mut R,
    ) -> Self {
        let shifted_rand = if has_degree_bound {
            Some(kzg10::Randomness::rand(hiding_bound, false, None, rng))
        } else {
            None
        };
        Self {
            rand: kzg10::Randomness::rand(hiding_bound, false, None, rng),
            shifted_rand,
        }
    }
}

impl<F: PrimeField, P: UVPolynomial<F>> Add for Randomness<F, P> {
    type Output = Self;

    #[inline]
    fn add(self, other: Self) -> Self {
        self + &other
    }
}

impl<F: PrimeField, P: UVPolynomial<F>> Zero for Randomness<F, P> {
    #[inline]
    fn zero() -> Self {
        Self { 
            rand: kzg10::Randomness::zero(),
            shifted_rand: None,
        }
    }

    #[inline]
    fn is_zero(&self) -> bool {
        unimplemented!()
    }
}

impl<F: PrimeField, P: UVPolynomial<F> + Share> Share for Randomness<F, P> {
    fn share<R: RngCore>(&self, num: usize, rng: &mut R) -> Vec<Self> {
        let rand_shares = self.rand.share(num, rng);
        if let Some(shifted_rand) = &self.shifted_rand {
            rand_shares
                .into_iter()
                .zip(shifted_rand.share(num, rng))
                .map(|(r, s)| Self { rand: r, shifted_rand: Some(s)})
                .collect()
        } else {
            rand_shares
                .into_iter()
                .map(|r| Self { rand: r, shifted_rand: None })
                .collect()
        }
    }
}

impl<F: PrimeField, P: UVPolynomial<F> + CanonicalSerialize> CanonicalSerialize for Randomness<F, P> {
    #[inline]
    fn serialize<W: Write>(&self, mut writer: W) -> Result<(), SerializationError> {
        self.rand.serialize(&mut writer)?;
        self.shifted_rand.serialize(&mut writer)?;
        Ok(())
    }

    #[inline]
    fn serialized_size(&self) -> usize {
        self.rand.serialized_size() + self.shifted_rand.serialized_size()
    }

    #[inline]
    fn serialize_uncompressed<W: Write>(&self, mut writer: W) -> Result<(), SerializationError> {
        self.rand.serialize_uncompressed(&mut writer)?;
        self.shifted_rand.serialize_uncompressed(&mut writer)?;
        Ok(())
    }

    #[inline]
    fn serialize_unchecked<W: Write>(&self, mut writer: W) -> Result<(), SerializationError> {
        self.rand.serialize_unchecked(&mut writer)?;
        self.shifted_rand.serialize_unchecked(&mut writer)?;
        Ok(())
    }

    #[inline]
    fn uncompressed_size(&self) -> usize {
        self.rand.uncompressed_size() + self.shifted_rand.uncompressed_size()
    }
}

impl<F: PrimeField, P: UVPolynomial<F> + CanonicalDeserialize> CanonicalDeserialize for Randomness<F, P> {
    #[inline]
    fn deserialize<R: Read>(mut reader: R) -> Result<Self, SerializationError> {
        let rand = kzg10::Randomness::<F, P>::deserialize(&mut reader)?;
        let shifted_rand = Option::<kzg10::Randomness::<F, P>>::deserialize(&mut reader)?;
        Ok(Self { rand, shifted_rand })
    }

    #[inline]
    fn deserialize_uncompressed<R: Read>(mut reader: R) -> Result<Self, SerializationError> {
        let rand = kzg10::Randomness::<F, P>::deserialize_uncompressed(&mut reader)?;
        let shifted_rand = Option::<kzg10::Randomness::<F, P>>::deserialize_uncompressed(&mut reader)?;
        Ok(Self { rand, shifted_rand })
    }

    #[inline]
    fn deserialize_unchecked<R: Read>(mut reader: R) -> Result<Self, SerializationError> {
        let rand = kzg10::Randomness::<F, P>::deserialize_unchecked(&mut reader)?;
        let shifted_rand = Option::<kzg10::Randomness::<F, P>>::deserialize_unchecked(&mut reader)?;
        Ok(Self { rand, shifted_rand })
    }
}
