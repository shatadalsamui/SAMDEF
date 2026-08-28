---
name: project-evaluation
description: Conducts an exhaustive, low-level technical evaluation of any software codebase, generating a comprehensive HTML and PDF grading report.
---

# Project Codebase & Architecture Evaluator

When activated, your task is to conduct an exhaustive, low-level technical evaluation of the current software engineering project. 

First, briefly assess the context of the project (e.g., academic capstone, startup prototype, enterprise tool, or open-source library) and adjust your baseline expectations accordingly. Evaluate the codebase as an advanced technical reviewer. Do not deduct marks for minor syntactical issues or trivial formatting preferences. Focus your grading on core systems engineering, architectural logic, code quality, and technical difficulty.

Follow these strict steps to complete the evaluation:

## Step 1: Systematic Discovery & Inspection (Exhaustive Deep-Dive)

**CRITICAL INSTRUCTION TO AGENT:** This is an intentionally long, exhaustive process. You are expected to consume a large context window. **Do not rush, do not skim, and do not infer logic just from file names.** You must read the actual contents of the code and documentation files. 

1. **Stack & Structure Mapping:**
   - Scan the root directory to identify the primary language(s), framework(s), and build tools.
   - Identify and **read through every single file** in the main source directories (e.g., `src/`, `app/`, `lib/`), documentation (`docs/`, `README.md`), and tests. 
   - **Do not skip any code or doc files** just because the directory is large. Take the time required to process them completely.
   - Strictly exclude build artifacts, dependencies (`node_modules`, `target`, `venv`), and environment files.

2. **Architecture & Pipeline Tracing:**
   - Trace the primary data flow and execution path line-by-line from the application's entry points.
   - Map out exactly how the project handles I/O, state management, and core processing. Do not hallucinate connections; verify them in the code.

3. **Technical Audit:**
   - Review memory management, concurrency models (threading/async), error handling, and performance considerations.
   - Verify security practices, separation of concerns, and modularity based on the actual implementation, not just the architectural claims in the README.

## Step 2: Scoring Rubric (100 Points Total)

Evaluate the project against the following generalized dimensions:

* **Systems & Software Architecture (30 pts):** Modularity, dataflow isolation, design patterns, separation of concerns, and component lifecycle management.
* **Technical Depth & Code Quality (20 pts):** Mastery of the chosen language/framework, error boundaries, memory safety, and performance optimizations.
* **Problem Solving & Logic (15 pts):** Algorithmic efficiency, handling of edge cases, and the complexity of the core business or processing logic.
* **Solution Viability & Reliability (15 pts):** Testing coverage/strategy, state predictability, scalability boundaries, and real-world robustness.
* **Toolchain Cohesion (10 pts):** Efficient use of build systems, package managers, CI/CD configurations, and deployment manifests.
* **Documentation & Readability (10 pts):** Clarity of inline comments, structural naming conventions, and quality of the README/setup instructions.

## Step 3: Required Evaluation Depth & Report Structure

Structure your final evaluation exactly into these sections:

1. **Executive Summary & Tech Stack Overview:**
   - A high-level summary of the project's purpose, the technologies used, and the overall architectural approach.
2. **Module-by-Module Technical Audit:**
   - Exhaustive analysis of the critical files and modules, covering data structures, compute bottlenecks, safety boundaries, and design choices.
3. **Itemized Scorecard with Point-by-Point Justifications:**
   - For every rubric dimension, provide an exact numerical score (e.g., `28/30`).
   - Explicitly list **every point awarded or deducted**, citing exact file names, functions, and line-level logic.
4. **Comprehensive Overall Feedback:**
   - Summarize key engineering achievements, primary technical debt or performance bottlenecks, and prioritized recommendations for refactoring or future iterations.
5. **Final Calculation:** Total the score properly out of 100.

## Step 4: Report & PDF Generation Pipeline

Generate the final deliverable strictly following this automated document pipeline. 

1. **Write `docs/Evaluation_Report.html`:**
   - Use the file writing tool to build a complete, self-contained HTML file (inline CSS only, no external dependencies) containing your full evaluation. If the `docs/` folder does not exist, create it first.
   - You MUST include the following print-ready CSS in the `<style>` tag:
     ```css
     @page { size: A4; margin: 18mm 15mm; }
     tr { page-break-inside: avoid; }
     thead { display: table-header-group; }
     pre, code { white-space: pre-wrap; word-break: break-word; }
     h1, h2, h3 { page-break-after: avoid; }
     body { font-family: system-ui, -apple-system, sans-serif; line-height: 1.6; color: #333; }
     table { border-collapse: collapse; width: 100%; margin-bottom: 20px; }
     th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }
     th { background-color: #f2f2f2; }
     ```

2. **Compile to PDF via WeasyPrint:**
   - Execute the following command using the terminal tool:
     ```bash
     python3 -m weasyprint docs/Evaluation_Report.html docs/Evaluation_Report.pdf
     ```
   - *(If the terminal output indicates WeasyPrint is missing, install it via `pip install --user --break-system-packages weasyprint` and rerun the PDF generation).*
