//! Conservative union coverage in collection, space, and acquisition time.
use crate::{error, manifest::Snapshot};
use stac::{api::Search, Bbox};
use superstac_core::errors::SuperSTACError;

#[derive(Clone, Copy)]
struct Region {
    x: [f64; 2],
    y: [f64; 2],
    t: [i128; 2],
}
fn region(bbox: Option<Bbox>, datetime: Option<&str>) -> Result<Region, SuperSTACError> {
    let (start, end) = datetime
        .map(stac::datetime::parse)
        .transpose()
        .map_err(error)?
        .unwrap_or_default();
    let nanos = |t: chrono::DateTime<chrono::FixedOffset>| {
        i128::from(t.timestamp()) * 1_000_000_000 + i128::from(t.timestamp_subsec_nanos())
    };
    Ok(Region {
        x: bbox.map_or([f64::NEG_INFINITY, f64::INFINITY], |b| [b.xmin(), b.xmax()]),
        y: bbox.map_or([f64::NEG_INFINITY, f64::INFINITY], |b| [b.ymin(), b.ymax()]),
        t: [start.map_or(i128::MIN, nanos), end.map_or(i128::MAX, nanos)],
    })
}
impl Region {
    fn subtract(mut self, other: Self, out: &mut Vec<Self>) {
        let intersection = Self {
            x: [self.x[0].max(other.x[0]), self.x[1].min(other.x[1])],
            y: [self.y[0].max(other.y[0]), self.y[1].min(other.y[1])],
            t: [self.t[0].max(other.t[0]), self.t[1].min(other.t[1])],
        };
        if intersection.x[0] > intersection.x[1]
            || intersection.y[0] > intersection.y[1]
            || intersection.t[0] > intersection.t[1]
        {
            out.push(self);
            return;
        }
        macro_rules! split {
            ($axis:ident) => {
                if self.$axis[0] < intersection.$axis[0] {
                    let mut part = self;
                    part.$axis[1] = intersection.$axis[0];
                    out.push(part);
                    self.$axis[0] = intersection.$axis[0];
                }
                if self.$axis[1] > intersection.$axis[1] {
                    let mut part = self;
                    part.$axis[0] = intersection.$axis[1];
                    out.push(part);
                    self.$axis[1] = intersection.$axis[1];
                }
            };
        }
        split!(x);
        split!(y);
        split!(t);
    }
}

pub(crate) fn covers(scopes: &[&Snapshot], search: &Search) -> Result<bool, SuperSTACError> {
    let bbox = if let Some(geometry) = &search.intersects {
        let mut item = stac::Item::new("bounds");
        item.set_geometry(geometry.clone()).map_err(error)?;
        item.bbox
    } else {
        search.items.bbox
    };
    // Do not infer a union for wrapping or 3D boxes.
    if bbox.is_some_and(|b| !b.is_valid() || !matches!(b, Bbox::TwoDimensional(_))) {
        return Ok(false);
    }
    let requested = region(bbox, search.items.datetime.as_deref())?;
    let collections: Vec<Option<&String>> = if search.collections.is_empty() {
        vec![None]
    } else {
        search.collections.iter().map(Some).collect()
    };
    for collection in collections {
        let mut missing = vec![requested];
        for scope in scopes {
            if !scope.scope.collections.is_empty()
                && collection.is_none_or(|c| !scope.scope.collections.contains(c))
            {
                continue;
            }
            let covered = region(scope.scope.bbox, scope.scope.datetime.as_deref())?;
            let mut next = Vec::new();
            for part in missing {
                part.subtract(covered, &mut next);
            }
            missing = next;
            if missing.is_empty() {
                break;
            }
            // Bound pathological fragmentation; falling back is safe.
            if missing.len() > 100_000 {
                return Ok(false);
            }
        }
        if !missing.is_empty() {
            return Ok(false);
        }
    }
    Ok(true)
}
