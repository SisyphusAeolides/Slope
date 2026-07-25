//! Bounded HTTPS lease types shared by Argus, Push, and Boulder's future
//! transport broker.
//!
//! These values are intentionally data-only. They do not expose a socket,
//! packet buffer, NIC register, or cryptographic key to a client. A broker
//! mints an [`HttpLease`] only after it has accepted the exact HTTPS origin and
//! bounded work budget; the client can then submit only matching requests over
//! a capability-validated IPC endpoint.

pub const MAX_HTTP_HOST_BYTES: usize = 48;
pub const MAX_HTTP_PATH_BYTES: usize = 128;
pub const MAX_HTTP_REQUEST_BYTES: u16 = 2_048;
pub const MAX_HTTP_RESPONSE_BYTES: u16 = 32 * 1024;
pub const MAX_TCP_SEGMENTS: u8 = 32;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HypermediaError {
    NotHttps,
    InvalidHost,
    InvalidPath,
    InvalidBudget,
    InvalidLease,
    LeaseExpired,
    LeaseMismatch,
}

/// Work admitted for one brokered HTTPS transaction. The transport must yield
/// after at most `yield_every_segments` reassembly steps, allowing the service
/// calculus to retain preemption points even during a TLS exchange.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HttpBudget {
    pub request_bytes: u16,
    pub response_bytes: u16,
    pub max_segments: u8,
    pub yield_every_segments: u8,
}

impl HttpBudget {
    pub const DEFAULT: Self = Self {
        request_bytes: 1_024,
        response_bytes: 16 * 1024,
        max_segments: 16,
        yield_every_segments: 1,
    };

    pub const fn is_valid(self) -> bool {
        self.request_bytes != 0
            && self.request_bytes <= MAX_HTTP_REQUEST_BYTES
            && self.response_bytes != 0
            && self.response_bytes <= MAX_HTTP_RESPONSE_BYTES
            && self.max_segments != 0
            && self.max_segments <= MAX_TCP_SEGMENTS
            && self.yield_every_segments != 0
            && self.yield_every_segments <= self.max_segments
    }
}

/// A canonical HTTPS origin. Host bytes are lower-level ASCII data so the
/// parser never allocates, normalizes Unicode, or consults ambient DNS.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HttpsOrigin {
    host: [u8; MAX_HTTP_HOST_BYTES],
    host_length: u8,
}

impl HttpsOrigin {
    pub fn new(host: &[u8]) -> Result<Self, HypermediaError> {
        if host.is_empty() || host.len() > MAX_HTTP_HOST_BYTES {
            return Err(HypermediaError::InvalidHost);
        }
        let mut previous_dot = true;
        for byte in host {
            let valid = byte.is_ascii_alphanumeric() || *byte == b'-' || *byte == b'.';
            if !valid || (*byte == b'.' && previous_dot) {
                return Err(HypermediaError::InvalidHost);
            }
            previous_dot = *byte == b'.';
        }
        if previous_dot || host[0] == b'-' || host[host.len() - 1] == b'-' {
            return Err(HypermediaError::InvalidHost);
        }
        let mut output = Self {
            host: [0; MAX_HTTP_HOST_BYTES],
            host_length: host.len() as u8,
        };
        for (index, byte) in host.iter().copied().enumerate() {
            output.host[index] = byte.to_ascii_lowercase();
        }
        Ok(output)
    }

    pub fn host(&self) -> &[u8] {
        &self.host[..usize::from(self.host_length)]
    }
}

/// One HTTPS request that can be sent only by a matching unexpired lease.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HttpsRequest {
    origin: HttpsOrigin,
    path: [u8; MAX_HTTP_PATH_BYTES],
    path_length: u8,
    budget: HttpBudget,
}

impl HttpsRequest {
    pub fn new(
        origin: HttpsOrigin,
        path: &[u8],
        budget: HttpBudget,
    ) -> Result<Self, HypermediaError> {
        if path.is_empty()
            || path.len() > MAX_HTTP_PATH_BYTES
            || path[0] != b'/'
            || !budget.is_valid()
        {
            return Err(if budget.is_valid() {
                HypermediaError::InvalidPath
            } else {
                HypermediaError::InvalidBudget
            });
        }
        if path
            .iter()
            .any(|byte| !byte.is_ascii_graphic() || *byte == b'#')
        {
            return Err(HypermediaError::InvalidPath);
        }
        let mut output = Self {
            origin,
            path: [0; MAX_HTTP_PATH_BYTES],
            path_length: path.len() as u8,
            budget,
        };
        output.path[..path.len()].copy_from_slice(path);
        Ok(output)
    }

    /// Parses only canonical `https://host/path` locations. Plain HTTP, user
    /// info, ports, fragments, and implicit schemes fail closed.
    pub fn parse_location(location: &[u8], budget: HttpBudget) -> Result<Self, HypermediaError> {
        let remainder = location
            .strip_prefix(b"https://")
            .ok_or(HypermediaError::NotHttps)?;
        let split = remainder
            .iter()
            .position(|byte| *byte == b'/')
            .unwrap_or(remainder.len());
        let origin = HttpsOrigin::new(&remainder[..split])?;
        let path = if split == remainder.len() {
            b"/".as_slice()
        } else {
            &remainder[split..]
        };
        Self::new(origin, path, budget)
    }

    pub const fn origin(&self) -> HttpsOrigin {
        self.origin
    }

    pub fn path(&self) -> &[u8] {
        &self.path[..usize::from(self.path_length)]
    }

    pub const fn budget(&self) -> HttpBudget {
        self.budget
    }
}

/// Opaque HTTPS authority imported from Push/Boulder. The raw capability is
/// private, so Argus cannot reinterpret it as a generic network capability.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HttpLease {
    capability: u64,
    generation: u32,
    origin: HttpsOrigin,
    budget: HttpBudget,
    expiry_epoch: u64,
}

impl HttpLease {
    /// Imports an exact broker-issued authority.
    ///
    /// # Safety
    ///
    /// Every field must come from Push's authenticated reply after Boulder has
    /// retained the corresponding transport/TLS state.
    pub const unsafe fn from_broker(
        capability: u64,
        generation: u32,
        origin: HttpsOrigin,
        budget: HttpBudget,
        expiry_epoch: u64,
    ) -> Result<Self, HypermediaError> {
        if capability == 0 || generation == 0 || expiry_epoch == 0 || !budget.is_valid() {
            return Err(HypermediaError::InvalidLease);
        }
        Ok(Self {
            capability,
            generation,
            origin,
            budget,
            expiry_epoch,
        })
    }

    pub fn permits(
        &self,
        request: HttpsRequest,
        current_epoch: u64,
    ) -> Result<(), HypermediaError> {
        if self.capability == 0 || self.generation == 0 {
            return Err(HypermediaError::InvalidLease);
        }
        if current_epoch >= self.expiry_epoch {
            return Err(HypermediaError::LeaseExpired);
        }
        if request.origin != self.origin
            || request.budget.request_bytes > self.budget.request_bytes
            || request.budget.response_bytes > self.budget.response_bytes
            || request.budget.max_segments > self.budget.max_segments
            || request.budget.yield_every_segments > self.budget.yield_every_segments
        {
            return Err(HypermediaError::LeaseMismatch);
        }
        Ok(())
    }

    pub const fn generation(&self) -> u32 {
        self.generation
    }

    pub const fn expiry_epoch(&self) -> u64 {
        self.expiry_epoch
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_canonical_https_locations_become_requests() {
        let request =
            HttpsRequest::parse_location(b"https://Example.COM/docs/start", HttpBudget::DEFAULT)
                .expect("HTTPS request");
        assert_eq!(request.origin().host(), b"example.com");
        assert_eq!(request.path(), b"/docs/start");
        assert_eq!(
            HttpsRequest::parse_location(b"http://example.com/", HttpBudget::DEFAULT),
            Err(HypermediaError::NotHttps)
        );
        assert!(
            HttpsRequest::parse_location(b"https://example.com:443/", HttpBudget::DEFAULT).is_err()
        );
    }

    #[test]
    fn lease_cannot_amplify_origin_budget_or_lifetime() {
        let origin = HttpsOrigin::new(b"example.com").expect("origin");
        let request = HttpsRequest::new(origin, b"/", HttpBudget::DEFAULT).expect("request");
        // SAFETY: the test uses a synthetic nonzero broker token.
        let lease = unsafe {
            HttpLease::from_broker(7, 3, origin, HttpBudget::DEFAULT, 10).expect("lease")
        };
        assert_eq!(lease.permits(request, 9), Ok(()));
        assert_eq!(
            lease.permits(request, 10),
            Err(HypermediaError::LeaseExpired)
        );

        let other = HttpsRequest::parse_location(b"https://other.example/", HttpBudget::DEFAULT)
            .expect("other request");
        assert_eq!(lease.permits(other, 1), Err(HypermediaError::LeaseMismatch));
    }
}
