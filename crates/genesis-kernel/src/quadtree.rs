//! Quadtree spatial partitioning for efficient spatial queries.
//!
//! This module provides a generic quadtree implementation for organizing objects
//! by their 2D bounding boxes, enabling O(log n) spatial queries.

/// Simple rectangle for bounds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    /// X coordinate of the left edge.
    pub x: f32,
    /// Y coordinate of the top edge.
    pub y: f32,
    /// Width of the rectangle.
    pub width: f32,
    /// Height of the rectangle.
    pub height: f32,
}

impl Rect {
    /// Creates a new rectangle.
    #[must_use]
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Creates a rectangle from center point and size.
    #[must_use]
    pub fn from_center(cx: f32, cy: f32, width: f32, height: f32) -> Self {
        Self {
            x: cx - width / 2.0,
            y: cy - height / 2.0,
            width,
            height,
        }
    }

    /// Returns the right edge x coordinate.
    #[must_use]
    pub fn right(&self) -> f32 {
        self.x + self.width
    }

    /// Returns the bottom edge y coordinate.
    #[must_use]
    pub fn bottom(&self) -> f32 {
        self.y + self.height
    }

    /// Returns the center point of the rectangle.
    #[must_use]
    pub fn center(&self) -> (f32, f32) {
        (self.x + self.width / 2.0, self.y + self.height / 2.0)
    }

    /// Checks if the rectangle contains a point.
    #[must_use]
    pub fn contains_point(&self, x: f32, y: f32) -> bool {
        x >= self.x && x < self.right() && y >= self.y && y < self.bottom()
    }

    /// Checks if this rectangle intersects with another.
    #[must_use]
    pub fn intersects(&self, other: &Rect) -> bool {
        self.x < other.right()
            && self.right() > other.x
            && self.y < other.bottom()
            && self.bottom() > other.y
    }

    /// Checks if this rectangle fully contains another.
    #[must_use]
    pub fn contains(&self, other: &Rect) -> bool {
        other.x >= self.x
            && other.right() <= self.right()
            && other.y >= self.y
            && other.bottom() <= self.bottom()
    }
}

/// Statistics about a quadtree.
#[derive(Debug, Clone, Default)]
pub struct QuadTreeStats {
    /// Total number of nodes in the tree.
    pub node_count: usize,
    /// Total number of objects stored.
    pub object_count: usize,
    /// Maximum depth of the tree.
    pub max_depth: usize,
    /// Number of leaf nodes.
    pub leaf_count: usize,
}

/// Quadtree node for spatial partitioning.
pub struct QuadTree<T> {
    /// Bounding rectangle of this node.
    bounds: Rect,
    /// Maximum objects before subdividing.
    max_objects: usize,
    /// Maximum tree depth.
    max_levels: usize,
    /// Current level (0 = root).
    level: usize,
    /// Objects stored in this node.
    objects: Vec<(Rect, T)>,
    /// Child nodes (NW, NE, SW, SE).
    children: Option<Box<[QuadTree<T>; 4]>>,
}

impl<T> QuadTree<T> {
    /// Creates a new quadtree with the given bounds.
    #[must_use]
    pub fn new(bounds: Rect, max_objects: usize, max_levels: usize) -> Self {
        Self {
            bounds,
            max_objects,
            max_levels,
            level: 0,
            objects: Vec::new(),
            children: None,
        }
    }

    /// Creates a child node at the specified level.
    fn new_child(bounds: Rect, max_objects: usize, max_levels: usize, level: usize) -> Self {
        Self {
            bounds,
            max_objects,
            max_levels,
            level,
            objects: Vec::new(),
            children: None,
        }
    }

    /// Returns the bounds of this node.
    #[must_use]
    pub const fn bounds(&self) -> &Rect {
        &self.bounds
    }

    /// Inserts an object with its bounding rect.
    ///
    /// Returns `true` if the object was inserted, `false` if it's outside bounds.
    pub fn insert(&mut self, bounds: Rect, object: T) -> bool {
        // Check if object fits in this node
        if !self.bounds.intersects(&bounds) {
            return false;
        }

        // If we have children, try to insert into them
        if let Some(children) = &mut self.children {
            for child in children.iter_mut() {
                if child.bounds.contains(&bounds) {
                    return child.insert(bounds, object);
                }
            }
            // Object spans multiple children, store here
            self.objects.push((bounds, object));
            return true;
        }

        // Store in this node
        self.objects.push((bounds, object));

        // Subdivide if needed and allowed
        if self.objects.len() > self.max_objects && self.level < self.max_levels {
            self.subdivide();

            // Re-insert objects that fit entirely in children
            let mut i = 0;
            while i < self.objects.len() {
                let mut moved = false;
                if let Some(children) = &mut self.children {
                    for child in children.iter_mut() {
                        if child.bounds.contains(&self.objects[i].0) {
                            let (rect, obj) = self.objects.swap_remove(i);
                            child.insert(rect, obj);
                            moved = true;
                            break;
                        }
                    }
                }
                if !moved {
                    i += 1;
                }
            }
        }

        true
    }

    /// Subdivides this node into 4 children.
    fn subdivide(&mut self) {
        let half_w = self.bounds.width / 2.0;
        let half_h = self.bounds.height / 2.0;
        let x = self.bounds.x;
        let y = self.bounds.y;
        let next_level = self.level + 1;

        self.children = Some(Box::new([
            // NW
            Self::new_child(
                Rect::new(x, y, half_w, half_h),
                self.max_objects,
                self.max_levels,
                next_level,
            ),
            // NE
            Self::new_child(
                Rect::new(x + half_w, y, half_w, half_h),
                self.max_objects,
                self.max_levels,
                next_level,
            ),
            // SW
            Self::new_child(
                Rect::new(x, y + half_h, half_w, half_h),
                self.max_objects,
                self.max_levels,
                next_level,
            ),
            // SE
            Self::new_child(
                Rect::new(x + half_w, y + half_h, half_w, half_h),
                self.max_objects,
                self.max_levels,
                next_level,
            ),
        ]));
    }

    /// Queries all objects that intersect with the given range.
    #[must_use]
    pub fn query(&self, range: Rect) -> Vec<&T> {
        let mut result = Vec::new();
        self.query_internal(&range, &mut result);
        result
    }

    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn query_internal<'a>(&'a self, range: &Rect, result: &mut Vec<&'a T>) {
        if !self.bounds.intersects(range) {
            return;
        }

        // Check objects at this level
        for (bounds, object) in &self.objects {
            if bounds.intersects(range) {
                result.push(object);
            }
        }

        // Recurse into children
        if let Some(children) = &self.children {
            for child in children.iter() {
                child.query_internal(range, result);
            }
        }
    }

    /// Queries objects with their bounds.
    #[must_use]
    pub fn query_with_bounds(&self, range: Rect) -> Vec<(&Rect, &T)> {
        let mut result = Vec::new();
        self.query_with_bounds_internal(&range, &mut result);
        result
    }

    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn query_with_bounds_internal<'a>(&'a self, range: &Rect, result: &mut Vec<(&'a Rect, &'a T)>) {
        if !self.bounds.intersects(range) {
            return;
        }

        // Check objects at this level
        for (bounds, object) in &self.objects {
            if bounds.intersects(range) {
                result.push((bounds, object));
            }
        }

        // Recurse into children
        if let Some(children) = &self.children {
            for child in children.iter() {
                child.query_with_bounds_internal(range, result);
            }
        }
    }

    /// Queries all objects at a point.
    #[must_use]
    pub fn query_point(&self, x: f32, y: f32) -> Vec<&T> {
        let mut result = Vec::new();
        self.query_point_internal(x, y, &mut result);
        result
    }

    fn query_point_internal<'a>(&'a self, x: f32, y: f32, result: &mut Vec<&'a T>) {
        if !self.bounds.contains_point(x, y) {
            return;
        }

        for (bounds, object) in &self.objects {
            if bounds.contains_point(x, y) {
                result.push(object);
            }
        }

        if let Some(children) = &self.children {
            for child in children.iter() {
                child.query_point_internal(x, y, result);
            }
        }
    }

    /// Clears all objects from the tree.
    pub fn clear(&mut self) {
        self.objects.clear();
        self.children = None;
    }

    /// Returns statistics about the tree.
    #[must_use]
    pub fn stats(&self) -> QuadTreeStats {
        let mut stats = QuadTreeStats::default();
        self.collect_stats(&mut stats, 0);
        stats
    }

    fn collect_stats(&self, stats: &mut QuadTreeStats, depth: usize) {
        stats.node_count += 1;
        stats.object_count += self.objects.len();
        stats.max_depth = stats.max_depth.max(depth);

        if let Some(children) = &self.children {
            for child in children.iter() {
                child.collect_stats(stats, depth + 1);
            }
        } else {
            stats.leaf_count += 1;
        }
    }

    /// Returns the number of objects in this node (not including children).
    #[must_use]
    pub fn len(&self) -> usize {
        self.objects.len()
    }

    /// Returns true if this node has no objects.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.objects.is_empty()
    }

    /// Returns true if this node has children.
    #[must_use]
    pub fn has_children(&self) -> bool {
        self.children.is_some()
    }
}

impl<T: Clone> QuadTree<T> {
    /// Removes all objects that match a predicate and returns them.
    pub fn remove_where<F>(&mut self, predicate: F) -> Vec<T>
    where
        F: Fn(&T) -> bool + Copy,
    {
        let mut removed = Vec::new();

        // Remove from this node
        let mut i = 0;
        while i < self.objects.len() {
            if predicate(&self.objects[i].1) {
                let (_, obj) = self.objects.swap_remove(i);
                removed.push(obj);
            } else {
                i += 1;
            }
        }

        // Recurse into children
        if let Some(children) = &mut self.children {
            for child in children.iter_mut() {
                removed.extend(child.remove_where(predicate));
            }
        }

        removed
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for QuadTree<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QuadTree")
            .field("bounds", &self.bounds)
            .field("level", &self.level)
            .field("objects", &self.objects.len())
            .field("has_children", &self.children.is_some())
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_rect_contains_point() {
        let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        assert!(rect.contains_point(50.0, 50.0));
        assert!(rect.contains_point(0.0, 0.0));
        assert!(!rect.contains_point(100.0, 100.0)); // Edge case: exclusive
        assert!(!rect.contains_point(-1.0, 50.0));
    }

    #[test]
    fn test_rect_intersects() {
        let a = Rect::new(0.0, 0.0, 100.0, 100.0);
        let b = Rect::new(50.0, 50.0, 100.0, 100.0);
        let c = Rect::new(200.0, 200.0, 50.0, 50.0);

        assert!(a.intersects(&b));
        assert!(b.intersects(&a));
        assert!(!a.intersects(&c));
    }

    #[test]
    fn test_rect_contains() {
        let outer = Rect::new(0.0, 0.0, 100.0, 100.0);
        let inner = Rect::new(25.0, 25.0, 50.0, 50.0);
        let partial = Rect::new(50.0, 50.0, 100.0, 100.0);

        assert!(outer.contains(&inner));
        assert!(!outer.contains(&partial));
        assert!(!inner.contains(&outer));
    }

    #[test]
    fn test_rect_from_center() {
        let rect = Rect::from_center(50.0, 50.0, 20.0, 20.0);
        assert!((rect.x - 40.0).abs() < f32::EPSILON);
        assert!((rect.y - 40.0).abs() < f32::EPSILON);
        assert!((rect.width - 20.0).abs() < f32::EPSILON);
        assert!((rect.height - 20.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_quadtree_insert_single() {
        let mut tree = QuadTree::new(Rect::new(0.0, 0.0, 1000.0, 1000.0), 4, 8);
        let inserted = tree.insert(Rect::new(100.0, 100.0, 10.0, 10.0), "object1");
        assert!(inserted);
        assert_eq!(tree.stats().object_count, 1);
    }

    #[test]
    fn test_quadtree_insert_outside_bounds() {
        let mut tree = QuadTree::new(Rect::new(0.0, 0.0, 100.0, 100.0), 4, 8);
        let inserted = tree.insert(Rect::new(200.0, 200.0, 10.0, 10.0), "outside");
        assert!(!inserted);
        assert_eq!(tree.stats().object_count, 0);
    }

    #[test]
    fn test_quadtree_subdivide() {
        let mut tree = QuadTree::new(Rect::new(0.0, 0.0, 1000.0, 1000.0), 2, 8);

        // Insert enough objects to trigger subdivision
        tree.insert(Rect::new(10.0, 10.0, 5.0, 5.0), "nw1");
        tree.insert(Rect::new(20.0, 20.0, 5.0, 5.0), "nw2");
        tree.insert(Rect::new(30.0, 30.0, 5.0, 5.0), "nw3");

        assert!(tree.has_children());
        assert_eq!(tree.stats().object_count, 3);
    }

    #[test]
    fn test_quadtree_query() {
        let mut tree = QuadTree::new(Rect::new(0.0, 0.0, 1000.0, 1000.0), 4, 8);

        tree.insert(Rect::new(100.0, 100.0, 10.0, 10.0), "a");
        tree.insert(Rect::new(500.0, 500.0, 10.0, 10.0), "b");
        tree.insert(Rect::new(900.0, 900.0, 10.0, 10.0), "c");

        // Query area containing only "a"
        let results = tree.query(Rect::new(0.0, 0.0, 200.0, 200.0));
        assert_eq!(results.len(), 1);
        assert_eq!(*results[0], "a");

        // Query entire area
        let all = tree.query(Rect::new(0.0, 0.0, 1000.0, 1000.0));
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn test_quadtree_query_point() {
        let mut tree = QuadTree::new(Rect::new(0.0, 0.0, 1000.0, 1000.0), 4, 8);

        tree.insert(Rect::new(100.0, 100.0, 50.0, 50.0), "a");
        tree.insert(Rect::new(120.0, 120.0, 50.0, 50.0), "b"); // Overlaps with "a"

        let at_overlap = tree.query_point(130.0, 130.0);
        assert_eq!(at_overlap.len(), 2);

        let outside = tree.query_point(0.0, 0.0);
        assert!(outside.is_empty());
    }

    #[test]
    fn test_quadtree_clear() {
        let mut tree = QuadTree::new(Rect::new(0.0, 0.0, 1000.0, 1000.0), 2, 8);

        for i in 0..10 {
            tree.insert(Rect::new(i as f32 * 50.0, i as f32 * 50.0, 10.0, 10.0), i);
        }

        assert!(tree.stats().object_count > 0);
        tree.clear();
        assert_eq!(tree.stats().object_count, 0);
        assert!(!tree.has_children());
    }

    #[test]
    fn test_quadtree_stats() {
        let mut tree = QuadTree::new(Rect::new(0.0, 0.0, 1000.0, 1000.0), 2, 4);

        for i in 0..20 {
            let x = (i % 10) as f32 * 100.0;
            let y = (i / 10) as f32 * 100.0;
            tree.insert(Rect::new(x, y, 5.0, 5.0), i);
        }

        let stats = tree.stats();
        assert_eq!(stats.object_count, 20);
        assert!(stats.node_count > 1);
        assert!(stats.max_depth > 0);
    }

    #[test]
    fn test_quadtree_10000_objects_query_performance() {
        let mut tree = QuadTree::new(Rect::new(0.0, 0.0, 10000.0, 10000.0), 16, 10);

        // Insert 10,000 objects
        for i in 0..10000 {
            let x = (i % 100) as f32 * 100.0;
            let y = (i / 100) as f32 * 100.0;
            tree.insert(Rect::new(x, y, 5.0, 5.0), i);
        }

        let stats = tree.stats();
        assert_eq!(stats.object_count, 10000);

        // Query a visible area (like a camera view)
        let query_rect = Rect::new(2000.0, 2000.0, 1920.0, 1080.0);

        let start = Instant::now();
        let _results = tree.query(query_rect);
        let elapsed = start.elapsed();

        // Should complete in < 1ms
        assert!(
            elapsed.as_millis() < 1,
            "Query took {}ms, expected < 1ms",
            elapsed.as_millis()
        );
    }

    #[test]
    fn test_quadtree_query_with_bounds() {
        let mut tree = QuadTree::new(Rect::new(0.0, 0.0, 1000.0, 1000.0), 4, 8);

        let rect_a = Rect::new(100.0, 100.0, 10.0, 10.0);
        let rect_b = Rect::new(500.0, 500.0, 10.0, 10.0);

        tree.insert(rect_a, "a");
        tree.insert(rect_b, "b");

        let results = tree.query_with_bounds(Rect::new(0.0, 0.0, 200.0, 200.0));
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0.x, 100.0);
        assert_eq!(*results[0].1, "a");
    }

    #[test]
    fn test_quadtree_remove_where() {
        let mut tree = QuadTree::new(Rect::new(0.0, 0.0, 1000.0, 1000.0), 4, 8);

        tree.insert(Rect::new(100.0, 100.0, 10.0, 10.0), 1);
        tree.insert(Rect::new(200.0, 200.0, 10.0, 10.0), 2);
        tree.insert(Rect::new(300.0, 300.0, 10.0, 10.0), 3);

        let removed = tree.remove_where(|&v| v % 2 == 0);
        assert_eq!(removed.len(), 1);
        assert_eq!(removed[0], 2);
        assert_eq!(tree.stats().object_count, 2);
    }

    #[test]
    fn test_quadtree_spanning_multiple_children() {
        let mut tree = QuadTree::new(Rect::new(0.0, 0.0, 100.0, 100.0), 2, 4);

        // Insert small objects to trigger subdivision
        tree.insert(Rect::new(10.0, 10.0, 5.0, 5.0), "small1");
        tree.insert(Rect::new(20.0, 20.0, 5.0, 5.0), "small2");
        tree.insert(Rect::new(30.0, 30.0, 5.0, 5.0), "small3");

        // Insert a large object that spans multiple quadrants
        tree.insert(Rect::new(40.0, 40.0, 30.0, 30.0), "spanning");

        // Query should find the spanning object
        let results = tree.query(Rect::new(45.0, 45.0, 10.0, 10.0));
        assert!(results.contains(&&"spanning"));
    }
}
