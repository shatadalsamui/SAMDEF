use crate::app::SamdefApp;
use crate::model::{class_name, DisplayDetection, CLASS_COUNT};
use gpui::ScrollStrategy;

/// Snapshot key identifying all inputs that affect the filtered detection table rows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableFilterKey {
    pub dataset_generation: u64,
    pub class_visible: [bool; CLASS_COUNT],
    pub confidence_bits: u32,
    pub search_query: String,
}

impl SamdefApp {
    /// Rebuild `table_filtered_indices` only when dataset, visibility filters, or query change.
    pub fn sync_table_filter_cache(&mut self) {
        let search_lower = self.table_search_query.trim().to_lowercase();
        let current_key = TableFilterKey {
            dataset_generation: self.dataset_generation,
            class_visible: self.class_visible,
            confidence_bits: self.confidence_threshold.to_bits(),
            search_query: search_lower,
        };
        if self.table_filter_key.as_ref() == Some(&current_key) {
            return;
        }

        filter_detection_indices_into(
            &self.detections,
            &self.class_visible,
            self.confidence_threshold,
            &current_key.search_query,
            &mut self.table_filtered_indices,
        );
        self.table_filter_key = Some(current_key);
        self.table_scroll_handle
            .scroll_to_item(0, ScrollStrategy::Top);
    }
}

/// Match decimal digits of `id` against an already-trimmed search query without heap allocation.
#[inline]
fn id_contains_digits(mut id: usize, needle: &str) -> bool {
    let nb = needle.as_bytes();
    if nb.is_empty() || !nb.iter().all(|b| b.is_ascii_digit()) {
        return false;
    }
    let mut buf = [0u8; 20];
    let mut i = buf.len();
    loop {
        i -= 1;
        buf[i] = b'0' + (id % 10) as u8;
        id /= 10;
        if id == 0 {
            break;
        }
    }
    buf[i..].windows(nb.len()).any(|w| w == nb)
}

/// Populate `out` with indices into `detections` matching visibility, confidence, and `search_lower`.
pub fn filter_detection_indices_into(
    detections: &[DisplayDetection],
    class_visible: &[bool; CLASS_COUNT],
    confidence_threshold: f32,
    search_lower: &str,
    out: &mut Vec<usize>,
) {
    let has_search = !search_lower.is_empty();
    let class_matches: [bool; CLASS_COUNT] = std::array::from_fn(|cid| {
        !has_search || class_name(cid).to_lowercase().contains(search_lower)
    });
    let numeric_search = !has_search || search_lower.bytes().all(|b| b.is_ascii_digit());

    out.clear();
    for (idx, d) in detections.iter().enumerate() {
        if d.class_id >= CLASS_COUNT
            || !class_visible[d.class_id]
            || d.confidence < confidence_threshold
        {
            continue;
        }
        if has_search
            && !class_matches[d.class_id]
            && !(numeric_search && id_contains_digits(d.id, search_lower))
        {
            continue;
        }
        out.push(idx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_det(id: usize, class_id: usize, confidence: f32) -> DisplayDetection {
        DisplayDetection {
            id,
            x_min: 0.0,
            y_min: 0.0,
            x_max: 10.0,
            y_max: 10.0,
            class_id,
            confidence,
        }
    }

    #[test]
    fn test_id_contains_digits() {
        assert!(id_contains_digits(0, "0"));
        assert!(id_contains_digits(1042, "04"));
        assert!(id_contains_digits(1042, "1042"));
        assert!(!id_contains_digits(1042, "99"));
        assert!(!id_contains_digits(1042, "1a"));
        assert!(!id_contains_digits(1042, ""));
    }

    #[test]
    fn test_filter_detection_indices_matches_visibility_conf_and_search() {
        let dets = vec![
            sample_det(0, 0, 0.90),
            sample_det(12, 1, 0.50),
            sample_det(120, 2, 0.10),
            sample_det(42, 0, 0.80),
        ];
        let mut vis = [true; CLASS_COUNT];
        let mut out = Vec::new();

        // Empty query filters by confidence & visibility only
        filter_detection_indices_into(&dets, &vis, 0.25, "", &mut out);
        assert_eq!(out, vec![0, 1, 3]);

        // Numeric query matches ID substring or class name
        filter_detection_indices_into(&dets, &vis, 0.25, "12", &mut out);
        assert_eq!(out, vec![1]);

        // Class name query matches case-insensitively
        let target_cls = class_name(0).to_lowercase();
        let partial = &target_cls[..target_cls.len().min(3)];
        filter_detection_indices_into(&dets, &vis, 0.25, partial, &mut out);
        assert_eq!(out, vec![0, 3]);

        // Class visibility toggle excludes matching class
        vis[0] = false;
        filter_detection_indices_into(&dets, &vis, 0.25, partial, &mut out);
        assert!(out.is_empty());
    }
}
