// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

//! Implementation of the KZG10 polynomial commitment scheme.
pub mod key;
pub mod srs;

pub use key::{CommitKey, OpeningKey};
pub use srs::PublicParameters;

use crate::transcript::TranscriptProtocol;
use crate::util::powers_of;
use dusk_bls12_381::{BlsScalar, G1Affine, G1Projective};
use merlin::Transcript;

#[derive(Copy, Clone, Debug)]
/// Proof that a polynomial `p` was correctly evaluated at a point `z`
/// producing the evaluated point p(z).
pub struct Proof {
    /// This is a commitment to the witness polynomial.
    pub commitment_to_witness: Commitment,
    /// This is the result of evaluating a polynomial at the point `z`.
    pub evaluated_point: BlsScalar,
    /// This is the commitment to the polynomial that you want to prove a statement about.
    pub commitment_to_polynomial: Commitment,
}

/// Proof that multiple polynomials were correctly evaluated at a point `z`,
/// each producing their respective evaluated points p_i(z).
#[derive(Debug)]
pub struct AggregateProof {
    /// This is a commitment to the aggregated witness polynomial.
    pub commitment_to_witness: Commitment,
    /// These are the results of the evaluating each polynomial at the point `z`.
    pub evaluated_points: Vec<BlsScalar>,
    /// These are the commitments to the polynomials which you want to prove a statement about.
    pub commitments_to_polynomials: Vec<Commitment>,
}

impl AggregateProof {
    /// Initialises an `AggregatedProof` with the commitment to the witness.
    pub fn with_witness(witness: Commitment) -> AggregateProof {
        AggregateProof {
            commitment_to_witness: witness,
            evaluated_points: Vec::new(),
            commitments_to_polynomials: Vec::new(),
        }
    }

    /// Adds an evaluated point with the commitment to the polynomial which produced it.
    pub fn add_part(&mut self, part: (BlsScalar, Commitment)) {
        self.evaluated_points.push(part.0);
        self.commitments_to_polynomials.push(part.1);
    }

    /// Flattens an `AggregateProof` into a `Proof`.
    /// The transcript must have the same view as the transcript that was used to aggregate the witness in the proving stage.
    pub fn flatten(&self, transcript: &mut Transcript) -> Proof {
        let challenge = transcript.challenge_scalar(b"aggregate_witness");
        let powers = powers_of(&challenge, self.commitments_to_polynomials.len() - 1);

        // Flattened polynomial commitments using challenge
        let flattened_poly_commitments: G1Projective = self
            .commitments_to_polynomials
            .iter()
            .zip(powers.iter())
            .map(|(poly, challenge)| poly.0 * challenge)
            .sum();
        // Flattened evaluation points
        let flattened_poly_evaluations: BlsScalar = self
            .evaluated_points
            .iter()
            .zip(powers.iter())
            .map(|(eval, challenge)| eval * challenge)
            .fold(BlsScalar::zero(), |acc, current_val| acc + current_val);

        Proof {
            commitment_to_witness: self.commitment_to_witness,
            evaluated_point: flattened_poly_evaluations,
            commitment_to_polynomial: Commitment::from_projective(flattened_poly_commitments),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
/// Holds a commitment to a polynomial in a form of a `G1Affine` Bls12_381 point.
pub struct Commitment(
    /// The commitment is a group element.
    pub G1Affine,
);

impl Commitment {
    /// Builds a `Commitment` from a Bls12_381 `G1Projective` point.
    pub fn from_projective(g: G1Projective) -> Self {
        Self(g.into())
    }
    /// Builds a `Commitment` from a Bls12_381 `G1Affine` point.
    pub fn from_affine(g: G1Affine) -> Self {
        Self(g)
    }
    /// Builds an empty `Commitment` which is equivalent to the
    /// `G1Affine` identity point in Bls12_381.
    pub fn empty() -> Self {
        Commitment(G1Affine::identity())
    }
}

impl Default for Commitment {
    fn default() -> Self {
        Commitment::empty()
    }
}
