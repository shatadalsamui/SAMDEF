## Non-Maximum Suppression (NMS) Loop Structure & Complexity

### Loop Structure

| Loop Level         | Iterates Over                | Times Run (per detection/class)      | Nesting Description                                 |
|--------------------|-----------------------------|--------------------------------------|-----------------------------------------------------|
| Outer loop         | All detections in a class   | n (number of detections in class)    | Main loop                                           |
| 1st nested loop    | Neighbor x (3 grid cells)   | 3 (for each detection)               | Nested inside main loop                             |
| 2nd nested loop    | Neighbor y (3 grid cells)   | 3 (for each neighbor x)              | Nested inside 1st nested loop                       |
| Inner loop         | Detections in a grid cell   | k (small, usually constant)          | Nested inside both neighborhood loops (x and y)      |

- **Outer loop:** Runs once per detection (`n` times). This is the main loop.
- **1st nested loop:** For each detection, iterates over 3 neighbor x grid cells.
- **2nd nested loop:** For each neighbor x, iterates over 3 neighbor y grid cells (total 3x3 = 9 grid cells).
- **Inner loop:** For each grid cell, checks all detections in that cell (`k` times, typically small and finite). This loop is nested inside both neighborhood loops.

---

#### 3x3 Neighborhood Explanation

For each detection, the algorithm checks the grid cell it is in (**the center cell**) as well as the 8 directly adjacent cells (up, down, left, right, and diagonals), forming a 3x3 neighborhood. This ensures that overlapping detections near cell boundaries are also considered and suppressed if necessary.

**Visual:**

|      |      |      |
|------|------|------|
| (gx-1,gy-1) | (gx,gy-1) | (gx+1,gy-1) |
| (gx-1,gy)   | (gx,gy)   | (gx+1,gy)   |
| (gx-1,gy+1) | (gx,gy+1) | (gx+1,gy+1) |

---

### Complexity Analysis

- **Total iterations:** ≈ n × 9 × k
    - `n`: Number of detections in the class
    - `9`: Number of neighboring grid cells (constant)
    - `k`: Detections per grid cell (small, nearly constant with good grid size)
- **Effective complexity:** **O(n)** (linear), since 9 and k are constants.

#### Why O(n)?
- Each detection only compares with a small, fixed number of neighbors (not all other detections).
- The grid structure ensures the inner loop remains small, avoiding the O(n²) cost of naive NMS.

---

**Summary:**  
Grid-based NMS is highly efficient: for n detections, the suppression step is O(n) in practice, thanks to constant-time neighborhood checks enabled by spatial partitioning.