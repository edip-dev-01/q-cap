# **Risks & Mitigations**

*(Normative)*

This section identifies technical, operational, and governance risks associated with Q-Cap capsules and defines required or recommended mitigations.
Where applicable, the specification mandates behaviors using **MUST**, **SHOULD**, and **MAY** semantics.



## **Immutability & Version Proliferation**

### **Risk**

Q-Capsules are immutable by design. Any modification results in a new Merkle root and a new capsule. This may lead to proliferation of capsule versions, complicating governance and distribution.

### **Mitigations**

* Capsule minting tools **SHOULD** embed explicit `version`, `parent_root`, and `created_at` metadata in the Provenance TLV.
* Producers **SHOULD** maintain external or in-capsule lineage manifests.
* Consumer tools **MUST** display version lineage when present.



## **Capability Token Mismanagement**

### **Risk**

Loss, duplication, or mishandling of capability tokens may grant unintended access.

### **Mitigations**

* Capability tokens **MUST** be cryptographically tied to the capsule’s Merkle root via the `qcap_root` field.
* Tokens **MUST** include expiration timestamps.
* Tokens **SHOULD** be signed only by issuer keys stored within hardware-backed secure environments (e.g., HSMs or TPMs).
* Capsules **MUST** reject tokens whose signatures do not validate against Trust Anchors.



## **Compromise of Issuer Keys**

### **Risk**

If an issuer’s private key is compromised, unauthorized tokens may be minted.

### **Mitigations**

* Capsules **SHOULD** list Trust Anchors with expiry or revocation metadata.
* Trust Anchor rotation **MUST** occur via append-only TLV updates.
* Consumers **MUST** fail verification on Trust Anchors marked as expired or revoked.
* Implementations **SHOULD** support threshold signing (multi-party approval) for high-sensitivity issuers.



## **Capsule Size & Resource Constraints**

### **Risk**

Large data shards, indexes, and provenance logs may increase capsule size beyond what is practical for edge or robotic environments.

### **Mitigations**

* Producers **SHOULD** allow generation of selective sub-views to minimize payload size.
* Data shards **SHOULD** employ columnar compression (zstd/gdeflate).
* Vector Indexes **MAY** use PQ/INT8 quantization by default.
* Consumers **SHOULD** support streaming/memory-mapped reads.



## **Edge Compute Limitations**

### **Risk**

Devices with constrained compute or memory may be unable to run WASM Lenses or kNN search efficiently.

### **Mitigations**

* Lenses **MUST** declare memory requirements and expected compute cost via the LensInfo ABI.
* Runtimes **MAY** reject Lenses exceeding device capability.
* Producers **SHOULD** provide lightweight Lens variants for tactical systems.
* Vector Index manifests **SHOULD** support reduced-dimension embeddings for constrained devices.



## **Misconfigured Policies or Selectors**

### **Risk**

Poorly defined Policy Graph rules or overly broad selectors may unintentionally reveal sensitive information.

### **Mitigations**

* Policy Graphs **SHOULD** undergo static validation during capsule minting.
* Producers **SHOULD** apply safe defaults (deny-by-default) for export and high-risk operations.
* Lenses that output sensitive features **MUST** consume privacy budget where applicable.
* Tools **MUST** warn when selectors match larger-than-intended subsets.



## **Differential Privacy (DP) Budget Exhaustion**

### **Risk**

Repeated DP queries may deplete privacy budgets and block further operations.

### **Mitigations**

* Privacy Ledger entries **MUST** accurately record epsilon/delta consumption.
* DP-enabled Lenses **MUST** fail deterministically when budgets are exhausted.
* Producers **SHOULD** allocate DP budgets appropriate to intended capsule lifetime.
* Auditors **MAY** reset or replenish budgets only through minting a new capsule.



## **Ecosystem Interoperability**

### **Risk**

Without widely available runtimes, capsules may not integrate easily with GIS, ML, or database environments.

### **Mitigations**

* Reference SDKs **SHOULD** be provided for major languages (Rust, Python, C++).
* Producers **SHOULD** supply a machine-readable Schema Block compatible with Arrow/Parquet metadata.
* Consumer tools **MAY** implement auto-conversion to interoperable formats (Arrow IPC, GeoParquet).



## **Cryptographic Vulnerabilities & Algorithm Agility**

### **Risk**

Incorrect crypto implementations, deprecated algorithms, or unmaintained PQ schemes may weaken capsule guarantees.

### **Mitigations**

* Capsules **MUST** state their cryptographic profile (e.g., FIPS, performance, PQ-enabled) in FLAGS.
* Hashing **MUST** use BLAKE3 (default) or SHA-256/SHA-3 (FIPS profile).
* Signatures **MUST** use Ed25519 or ECDSA P-256, optionally combined with ML-DSA.
* Key Encapsulation Mechanisms **SHOULD** support hybrid classical + ML-KEM.
* Implementations **SHOULD** support algorithm agility through profile negotiation.



## **Sub-View Trust Boundary Reset**

### **Risk**

Users may incorrectly assume that capability tokens apply to derived capsules.

### **Mitigations**

* Sub-views **MUST** generate a new Merkle root and new Trust Anchor space.
* Parent capability tokens **MUST NOT** apply to sub-views.
* Minting tools **MUST** warn users when generating sub-views without embedding new Trust Anchors or local policy.



## **Weak Upstream Data Governance**

### **Risk**

Capsule integrity does not guarantee correctness or legality of the underlying source data.

### **Mitigations**

* Provenance TLV **SHOULD** contain in-toto style records for preprocessing pipelines.
* Lenses **MAY** include data quality audits.
* Capsule producers **SHOULD** follow organizational data governance and classification policies before minting.



## **Human Factors & Training Requirements**

### **Risk**

Q-Cap introduces unfamiliar concepts (e.g., Merkle DAGs, DP ledgers, policy graphs).

### **Mitigations**

* Implementations **SHOULD** provide UX-friendly inspection tools.
* Training materials **SHOULD** accompany operational deployments.
* High-level capsule summaries **SHOULD** be available via `qcap describe`.
* Organizations **SHOULD** define SOPs for minting, issuing capabilities, and sub-view governance.



## **Environmental & Tactical Constraints**

### **Risk**

In robotic, drone, or battlefield environments, device failure, partial downloads, or hostile tampering may corrupt capsules.

### **Mitigations**

* Capsules **MUST** verify Merkle roots on ingest.
* Implementations **MAY** support shard-level error correction (Reed-Solomon).
* Disconnected nodes **SHOULD** verify Trust Anchors and integrity before mission loading.
* Devices **SHOULD** reject incomplete or partially transmitted capsules.



# **Summary**

Q-Cap’s design introduces new capabilities and new governance responsibilities.
The mitigations above ensure capsules remain:

* secure
* verifiable
* privacy-respecting
* policy-enforcing
* suitable for contested, air-gapped, and multi-authority environments

…while minimizing operational and cryptographic risks.
