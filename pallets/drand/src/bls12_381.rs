/*
 * Copyright 2024 by Ideal Labs, LLC
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use crate::utils::ScalarFieldFor;
use ark_bls12_381::{G1Affine, G1Projective, G2Affine, G2Projective};
use ark_ec::{pairing::Pairing, short_weierstrass::SWCurveConfig, AffineRepr};
use sp_ark_bls12_381::{
    Bls12_381 as Bls12_381Opt, G1Affine as G1AffineOpt, G1Projective as G1ProjectiveOpt,
    G2Affine as G2AffineOpt, G2Projective as G2ProjectiveOpt,
};

#[inline]
pub fn pairing(a: G1Affine, b: G2Affine) {
    let _out = ark_bls12_381::Bls12_381::multi_pairing([a], [b]);
}

#[inline]
pub fn pairing_opt(a: G1AffineOpt, b: G2AffineOpt) {
    let _out = Bls12_381Opt::multi_pairing([a], [b]);
}

#[inline]
pub fn msm_g1(bases: &[G1Affine], scalars: &[ScalarFieldFor<G1Affine>]) {
    let _out = <ark_bls12_381::g1::Config as SWCurveConfig>::msm(bases, scalars);
}

#[inline]
pub fn msm_g1_opt(bases: &[G1AffineOpt], scalars: &[<G1AffineOpt as AffineRepr>::ScalarField]) {
    let _out = <sp_ark_bls12_381::g1::Config as SWCurveConfig>::msm(bases, scalars);
}

#[inline]
pub fn msm_g2(bases: &[G2Affine], scalars: &[ScalarFieldFor<G2Affine>]) {
    let _out = <ark_bls12_381::g2::Config as SWCurveConfig>::msm(bases, scalars);
}

#[inline]
pub fn msm_g2_opt(bases: &[G2AffineOpt], scalars: &[ScalarFieldFor<G2AffineOpt>]) {
    let _out = <sp_ark_bls12_381::g2::Config as SWCurveConfig>::msm(bases, scalars);
}

#[inline]
pub fn mul_projective_g1(base: &G1Projective, scalar: &[u64]) {
    let _out = <ark_bls12_381::g1::Config as SWCurveConfig>::mul_projective(base, scalar);
}

#[inline]
pub fn mul_projective_g1_opt(base: &G1ProjectiveOpt, scalar: &[u64]) {
    let _out = <sp_ark_bls12_381::g1::Config as SWCurveConfig>::mul_projective(base, scalar);
}

#[inline]
pub fn mul_affine_g1(base: &G1Affine, scalar: &[u64]) {
    let _out = <ark_bls12_381::g1::Config as SWCurveConfig>::mul_affine(base, scalar);
}

#[inline]
pub fn mul_affine_g1_opt(base: &G1AffineOpt, scalar: &[u64]) {
    let _out = <sp_ark_bls12_381::g1::Config as SWCurveConfig>::mul_affine(base, scalar);
}

#[inline]
pub fn mul_projective_g2(base: &G2Projective, scalar: &[u64]) {
    let _out = <ark_bls12_381::g2::Config as SWCurveConfig>::mul_projective(base, scalar);
}

#[inline]
pub fn mul_projective_g2_opt(base: &G2ProjectiveOpt, scalar: &[u64]) {
    let _out = <sp_ark_bls12_381::g2::Config as SWCurveConfig>::mul_projective(base, scalar);
}

#[inline]
pub fn mul_affine_g2(base: &G2Affine, scalar: &[u64]) {
    let _out = <ark_bls12_381::g2::Config as SWCurveConfig>::mul_affine(base, scalar);
}

#[inline]
pub fn mul_affine_g2_opt(base: &G2AffineOpt, scalar: &[u64]) {
    let _out = <sp_ark_bls12_381::g2::Config as SWCurveConfig>::mul_affine(base, scalar);
}
