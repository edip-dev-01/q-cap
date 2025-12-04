# **Decentralized Issuance Model**

*(Normative)*

## **Overview**

Q-Cap supports a decentralized authorization model in which **multiple independent entities** may issue capability tokens for a capsule.
Authorization does **not** depend on any centralized service, online infrastructure, or shared PKI.
Instead, each capsule embeds its own **Trust Anchors**, defining which issuers are authorized to sign tokens.

A capability token is valid **if and only if**:

1. It is signed by a private key whose corresponding public key appears in the capsule’s **Trust Anchors** TLV, and
2. The token’s `qcap_root` field matches the capsule’s Merkle root.

If either condition fails, the capsule must reject the token.

This model enables **federated, multi-agency, multi-party governance** while maintaining offline verifiability and strong cryptographic containment.

---

## **Trust Anchors (Authorized Issuers)**

Each Q-Cap contains a TLV section:

```
0x0001 – Trust Anchors
```

This section defines one or more COSE_Key objects representing authorized issuers.

Each Trust Anchor includes:

* **Public key material** (Ed25519, ECDSA P-256, ML-DSA)
* **Key identifier (kid)**
* **Optional metadata** (expiry, revocation hints, jurisdiction, role tags)

These keys represent the **set of issuers** permitted to mint capability tokens for this capsule.

### **Normative Requirement**

> A capsule MUST accept a capability token only if its COSE signature validates against one of the public keys declared in the Trust Anchors TLV.

---

## **Decentralized Issuance**

Any entity possessing a private key corresponding to a Trust Anchor MAY issue capability tokens.
Issuance does not require network connectivity, a central registry, or communication with other issuers.

### Examples of decentralized issuance:

* Multi-department collaboration (e.g., DND, NRCan, Public Safety each issue tokens).
* Role-based delegation inside a single organization.
* Field-level issuing authority in air-gapped or tactical environments.
* Federated research or international partnerships.

### Benefits

* No single point of control or failure
* No dependency on online authorization servers
* Works across air-gapped and cross-domain environments
* Perfectly suited for defense, emergency response, and distributed science

---

## **Token Binding to Capsules**

Each capability token MUST contain a `qcap_root` field:

```
"qcap_root": "blake3:<32-byte digest>"
```

This binds the token to **one specific capsule state**.

### **Normative Requirement**

> A capsule MUST reject any token whose `qcap_root` does not exactly match its own Merkle root.

Consequences:

* Tokens **cannot be reused** across modified or corrupted copies.
* Tokens **do not migrate** to sub-views or derived capsules unless explicitly re-issued.
* Replay attacks across unrelated capsules are cryptographically impossible.

---

## **Sub-Views and Issuer Scope**

A sub-view produced via `qcap reveal` results in a new capsule with a **new Merkle root**.

### Normative rules:

* Capability tokens for the parent capsule MUST NOT grant permissions to the sub-view.
* Sub-views MUST define their own Trust Anchors if further delegated issuance is allowed.
* New tokens for the sub-view MUST be minted explicitly, referencing the sub-view’s root.

This prevents capability “leakage” from high-fidelity to low-fidelity or public artifacts.

---

## **Governance Flexibility**

Issuing authorities may define their own governance structures, such as:

* A single “root” authority with delegated subordinate keys
* Multiple equal-authority issuers across organizations
* Role-based distinctions (e.g., “auditor”, “analyst”, “external partner”)
* Jurisdiction-bound issuers
* Time-limited or purpose-restricted issuers

The capsule does not mandate a governance model; it **enforces** the one declared in the Trust Anchors.

---

## **Key Rotation and Revocation**

Because Q-Cap capsules are immutable, trust-anchor rotation occurs through **append-only updates**:

1. Add new issuer keys via a TLV update.
2. Mark old keys as expired or revoked in Trust Anchor metadata.
3. Update the Merkle DAG and Signatures accordingly.

Verification tools MUST check Trust Anchor metadata for revocation/expiry markers.

---

## **Security Properties**

### **Offline Verifiability**

All issuer legitimacy checks occur offline using Trust Anchors embedded in the capsule.

### **Decentralized Trust**

No central directory or CA is required; capsules express their own trust boundaries.

### **Strong Containment**

Tokens cannot be misapplied to:

* other datasets,
* other versions, or
* derived capsules.

### **Resistance to Rogue Issuance**

A malicious party cannot issue valid tokens unless their public key is explicitly listed in Trust Anchors.

---

## **Summary**

The decentralized issuance model allows Q-Cap to function as a **self-governing, portable data artifact** that supports:

* Military and emergency offline operations
* Interdepartmental and coalition sharing
* Zero-trust, cross-domain workflows
* Multilateral scientific collaboration
* Distributed governance without centralized servers

This model is a key differentiator of Q-Cap relative to traditional file formats and platform-centric access control systems.
