//! Bounded route scoring shared with the optional Fortran policy kernel.

/// Evidence consumed when deciding whether a service route should be used.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RouteEvidence {
    pub authority: f64,
    pub liveness: f64,
    pub boundedness: f64,
    pub pressure: f64,
}

impl RouteEvidence {
    const fn as_array(self) -> [f64; 4] {
        [
            self.authority,
            self.liveness,
            self.boundedness,
            self.pressure,
        ]
    }
}

/// Scores an admitted route in `[0, 1]`.
///
/// Authority and bounded execution dominate the result. Resource pressure is
/// a penalty, so an otherwise healthy route backs away before saturation.
pub fn route_score(evidence: RouteEvidence) -> f64 {
    let values = evidence.as_array();
    route_score_impl(&values)
}

#[cfg(feature = "fortran-policy")]
fn route_score_impl(values: &[f64; 4]) -> f64 {
    unsafe extern "C" {
        fn arach_slope_route_score(features: *const f64, count: i32) -> f64;
    }

    // SAFETY: `values` contains four contiguous f64 values for the duration of
    // the call and the Fortran boundary reads exactly `count` elements.
    unsafe { arach_slope_route_score(values.as_ptr(), values.len() as i32) }
}

#[cfg(not(feature = "fortran-policy"))]
fn route_score_impl(values: &[f64; 4]) -> f64 {
    let authority = values[0].clamp(0.0, 1.0);
    let liveness = values[1].clamp(0.0, 1.0);
    let boundedness = values[2].clamp(0.0, 1.0);
    let pressure = values[3].clamp(0.0, 1.0);
    (authority * 0.35 + liveness * 0.25 + boundedness * 0.35 - pressure * 0.20).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn healthy_bounded_route_outranks_saturated_route() {
        let healthy = route_score(RouteEvidence {
            authority: 1.0,
            liveness: 1.0,
            boundedness: 1.0,
            pressure: 0.0,
        });
        let saturated = route_score(RouteEvidence {
            pressure: 1.0,
            ..RouteEvidence {
                authority: 1.0,
                liveness: 1.0,
                boundedness: 1.0,
                pressure: 0.0,
            }
        });
        assert!(healthy > saturated);
        assert!((0.0..=1.0).contains(&healthy));
        assert!((0.0..=1.0).contains(&saturated));
    }

    #[test]
    fn untrusted_route_is_not_admitted_by_score_alone() {
        let score = route_score(RouteEvidence {
            authority: 0.0,
            liveness: 1.0,
            boundedness: 1.0,
            pressure: 0.0,
        });
        assert!(score < 0.75);
    }
}
