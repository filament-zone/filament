# Cryptographic primitives

Various parts of Pulzaar require cryptographic primitives. We treat these as opaque
as much as we can and focus on the functionality they provide.

## Hash functions

Pulzaar uses `SHA-256` and `SHA-512` as defined in [FIPS 180-4](http://dx.doi.org/10.6028/NIST.FIPS.180-4).
Both `SHA-256` and `SHA-512` take as input an arbitrary array of bytes and
output 32bytes and 64bytes respectively.

- `SHA-256(m) -> d` takes message `m` and outputs digest `d` of length 32 bytes
- `SHA-512(m) -> d` takes message `m` and outputs digest `d` of length 64 bytes

## Authenticated data structure (ADS)

Authenticated data structures allow updates by untrusted operators which can be
verified efficiently by other parties. Merkle trees are one such data structure
that can be used as a mapping from arbitrary keys to values.

The functionality we are looking for:

- `insert(k, v) -> h` insert value `v` at key `k` and return new digest for the
  full data structure
- `lookup(k) -> (v, p)` lookup `k` and return `v` and a proof of inclusion `p`
  or return a default if no value found

In particular we use the [Jelly Merkle Tree](https://developers.diem.com/papers/jellyfish-merkle-tree/2021-01-14.pdf),
which is a sparse merkle tree where each internal node has up to 16 children.
The hash function we use must be collision resistant and it must be infeasible
to find a pre-image s.t. the digest is the default digest. `SHA-256` fits both
of these requirements and is what we use.

## Digital signatures (DS)

A digital signature scheme is defined by:

- signing key `sk` (also called secret or private key)
- verification key `vk` (also called public key `pk`)
- message `m`
- signature `s`
- three algorithms to
  - `gen() -> (sk, vk)`: generate `sk` and `vk`
  - `sign(sk, m) -> s`: sign message `m` with signing key `sk`
  - `verify(pk, m, s)`: verify signature `s` given `pk` and `m`

Pulzaar uses one signature scheme: `Ed25519`. This is also the scheme used by CometBFT
consensus clients, though verifying the validity of Ed25519 signatures is
[notoriously complex in practice](https://hdevalence.ca/blog/2020-10-04-its-25519am).
