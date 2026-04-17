# Architectural Decision Records

Every architectural decision in Nukm lands here as a numbered, dated markdown file. ADRs are append-only: amendments are new ADRs that supersede earlier ones, never edits.

## Format

```markdown
# ADR-NNNN: <title>

Date: YYYY-MM-DD
Status: Proposed | Accepted | Superseded by ADR-MMMM

## Context
<one paragraph: what problem or question>

## Decision
<one paragraph: what we chose>

## Consequences
<bullets: what this makes easy, what this makes hard, what we defer>
```

## Numbering

Zero-padded to four digits. `0001` through `9999`. Allocate the next free number when you start drafting; do not renumber after commit.

## When to write one

- Crate boundary or public API shape changes
- Choice of dependency that constrains downstream design (`git2`, `trash`, `tokio` flavour, ...)
- Test strategy (nextest adoption, proptest introduction, corpus structure)
- Platform trait or detector trait signature changes
- Any decision that future maintainers would ask "why" about

Ordinary refactors, bug fixes, and small additions do not need an ADR.

## Review

Arthur reviews every ADR before it lands.
