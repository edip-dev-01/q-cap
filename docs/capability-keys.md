Here is a practical explanation of **how someone gets a capability key** after downloading a Q-Cap capsule.

# Start With the Basic Idea

A **capability key (capability token)** is a small, signed permission slip that says:

> “Given this specific Q-Cap file, this person may see or do *exactly* these things.”

It is:

* **issued by the capsule’s owner/producer**
* **cryptographically bound to the capsule’s Merkle root**
* **checked locally by the Q-Cap toolset**

So the capsule itself enforces the permissions.

# In Practice: How Does a User Get One?

There are **three real-world paths**, depending on the deployment model.


## **Path 1 — The Producer Hands Out Capability Tokens**

This is the simplest and primary model.

### **Step 1 — User downloads the capsule file**

```
dataset.qcap
```

### **Step 2 — User requests access from the owner**

E.g., they email, submit a form, authenticate to a portal, etc.

### **Step 3 — The owner runs:**

```
qcap mint-cap dataset.qcap \
   --role partner_analyst \
   --allow knn:index_clip512 \
   --reveal columns:lat,lon,class \
   --expires 2025-12-31
```

This command generates something like:

```
cap_47af93a0.jsoncbor
```

### **Step 4 — The owner sends the capability token to the user**

The user stores it locally.

### **Step 5 — The user runs commands using the capability**

Example:

```
qcap reveal dataset.qcap --cap cap_47af93a0 > subset.qcap
```

or, to run a Lens:

```
qcap run-lens dataset.qcap clip-embed --cap cap_47af93a0
```

This is the **standard operational model**.


## **Path 2 — The Capsule Contains a Public-Access Policy**

Some Q-Caps intentionally contain “open” policy entries, like:

* Anyone can run kNN on the public index.
* Anyone can view non-sensitive columns.
* More sensitive operations still require a capability token.

This is useful for:

* Publicly released, non-sensitive views
* Academic or open-data contexts
* “Open capsule / closed capsule” dual publishing (from your academic paper)

In this case the user **does not need a capability token** for allowed actions.


## **Path 3 — There Is an Automated Authorization Service**

Nothing in Q-Cap requires it — but organizations can choose to have a server issue tokens automatically.

For example:

* A web portal that authenticates the user
* A SharePoint page with an embedded Q-Cap token issuer
* An internal “Data Access Broker”
* A STAC-like catalogue that hands out Q-Cap tokens per asset

The workflow becomes:

1. User downloads capsule
2. User logs into a controlled environment
3. System issues capability token based on role/group/purpose
4. User runs Q-Cap locally using that token

This is entirely optional.

# Why Capability Tokens Work So Well

Everything is bound together:

* The token is **signed** (COSE Sign1)
* The token includes **qcap_root**:

  ```
  "qcap_root": "blake3:abc123…"
  ```
* The capsule enforces access **offline**
* The **privacy ledger records** use of sensitive permissions
* Sub-views retain cryptographic linkage to the original

From the v0 spec:

> “Capability tokens (COSE-signed) describe selectors and actions; tokens bind to the capsule’s Merkle root.”
>

# A Concrete Realistic Scenario

Let’s say analyst Emma downloads:

```
orders_2025.qcap
```

She wants to:

* run a Lens called `force-readiness@1.0`
* export aggregated province-level counts

She asks the data owner.

Owner runs:

```
qcap mint-cap orders_2025.qcap \
   --role emma \
   --allow lens-run:force-readiness@1.0 \
   --allow aggregate:dp_count \
   --epsilon 0.5 \
   --expires 2026-03-31
```

Emma now receives:

```
emma_cap_2026.cbor
```

She can now do:

```
qcap run-lens orders_2025.qcap force-readiness@1.0 --cap emma_cap_2026.cbor
```

If she tries anything else (export raw rows, circumvent DP restrictions), the capsule denies it with:

```
ERROR: action not permitted by capability
```

No server. No firewall rules. The capsule enforces everything itself.

## Key Insight

**The capability token is not embedded inside the Q-Cap. It is a separate signed object delivered to the user.**

This separation:

* removes need for re-encryption
* avoids creating new variants of the capsule
* supports multi-party collaboration
* keeps access control auditable and portable
