---
name: project-evaluation2
description: Conducts a massive, file-by-file exhaustive low-level technical evaluation of a prototype software codebase, generating a comprehensive report.
---

# Project Codebase & Architecture Evaluator

When activated, your task is to conduct an exhaustive, granular, low-level technical evaluation of the current software engineering project. 

**CRITICAL PROTOTYPE INSTRUCTION:** Evaluate the codebase strictly as an advanced, high-performance prototype or proof-of-concept. **Do not evaluate this as a production-ready enterprise application.** Do not deduct marks for missing enterprise polish (e.g., CI/CD pipelines, advanced logging frameworks, high availability, or minor surface-level bug handling). Focus entirely on core systems engineering, architectural logic, and technical difficulty.

**CRITICAL FORMAT INSTRUCTION:** This is an intentionally massive, exhaustive process designed to generate a 10+ page PDF. **Do not rush, do not skim, and do not summarize modules.** You must read the actual contents of the code and dedicate a specific analysis block to every single file. 

## Step 1: Systematic Discovery & Context Calibration

1. **Context & Difficulty Calibration (CRITICAL):**
   - Assess the technical difficulty of the chosen stack. Actively look for and heavily reward custom, low-level engineering (e.g., native memory management, custom algorithms, bare-metal bindings) over the use of standard, high-level wrapper libraries. 
2. **Stack & Structure Mapping:**
   - Scan the root directory to identify the primary language(s), framework(s), and build tools.
   - Identify and **read through every single file** in the main source directories, documentation, and tests. 
   - **Do not skip any code or doc files.**
   - Strictly exclude build artifacts, dependencies, and environment files.
3. **Architecture & Pipeline Tracing:**
   - Trace the primary data flow and execution path line-by-line from the application's entry points.

## Step 2: Scoring Rubric (100 Points Total)

Evaluate the project against the following dimensions. **Grade relative to the project's scope as a prototype, heavily rewarding technical ambition:**

* **Systems & Software Architecture (25 pts):** Modularity, dataflow isolation, concurrency design, memory safety, and architectural foundation.
* **Technical Depth & Low-Level Implementation (25 pts):** Mastery of the chosen language, precision of core algorithms, bare-metal/runtime bindings, and performance optimizations.
* **Difficulty & Innovation (20 pts):** Bypassing standard/easy wrappers in favor of custom, high-throughput logic. Handling of complex edge cases and engineering ambition.
* **Prototype Viability & Proof of Concept (10 pts):** Does the core concept work efficiently? Bounded memory consumption, throughput scalability, and successful validation of the core idea. (Do not deduct for lack of production readiness).
* **Toolchain Cohesion (10 pts):** Synergy across the chosen stack and build systems.
* **Documentation (10 pts):** Clarity of inline comments, structural naming conventions, and setup instructions.

## Step 3: Required Evaluation Depth & Report Structure

Structure your final evaluation exactly into these sections. **Do not cut corners.**

1. **Executive Summary & Baseline Calibration:**
   - A high-level summary of the prototype's purpose and a direct comparison of its technical difficulty against standard projects in its category.
2. **STRICT File-by-File Low-Level Technical Audit (CORE REQUIREMENT):**
   - **You MUST create a dedicated subsection for EVERY SINGLE SOURCE FILE discovered.**
   - Do not group files. For each file, detail its data structures, memory layout, compute bottlenecks, and safety boundaries line-by-line.
3. **Itemized Scorecard with Point-by-Point Justifications:**
   - For every rubric dimension, provide an exact numerical score (e.g., `23/25`).
   - Explicitly list **every point awarded or deducted**, citing exact file names, functions, and line-level logic.
4. **Comprehensive Overall Feedback:**
   - Summarize key engineering achievements, technical debt, and prioritized recommendations for taking the prototype to the next stage.
5. **Final Calculation:** Total the score properly out of 100.

## Step 4: Report Generation Pipeline (Markdown)

Generate the final deliverable strictly following this documentation pipeline. 

1. **Write `docs/Evaluation_Report.md`:**
   - Use the file writing tool to build a comprehensive, beautifully structured, and highly detailed Markdown file containing your full evaluation. If the `docs/` folder does not exist, create it first.
   - Utilize advanced Markdown formatting to make the report visually appealing, professional, and easy to read. This includes:
     - Clear hierarchical headings (`#`, `##`, `###`) to separate major sections.
     - Bold and italic text for emphasis on key metrics, file names, and critical findings.
     - Bullet points and numbered lists for readability and structured breakdowns.
     - Markdown tables for the scoring rubric, tech stack overview, and point deductions.
     - Fenced code blocks with appropriate syntax highlighting (e.g., ````python`, ````bash`) for any code snippets, architecture diagrams, or terminal commands referenced.
     - Blockquotes (`>`) for executive summaries, key takeaways, or critical warnings.
     - Horizontal rules (`---`) to cleanly separate the major evaluation phases.
   - Ensure the document is entirely self-contained, exhaustively detailed, and formatted as a high-end technical grading report.
