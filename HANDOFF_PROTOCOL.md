# Handoff protocol

Every agent-to-agent handoff inside the Nukm repository follows this shape. Name the target agent explicitly at the top of the response initiating the handoff.

## Context

- Phase: `<number and name, e.g. Phase 1 - Core + CLI MVP>`
- Crate: `<nukm-core | nukm-cli | nukm-gui | cross-cutting>`
- File(s): `<paths relative to repo root>`

## State

- **Done:** bullet list of completed items
- **In progress:** bullet list of open items
- **Blocked:** bullet list, each with the reason for the block

## Request

A single specific artefact or decision requested from the receiving agent. If it is not single or not specific, the handoff is not ready.

## Acceptance criteria

- Measurable condition 1
- Measurable condition 2
- ...

## Return path

- **Return to:** `<agent name>`
- **Expected follow-up:** what happens after this handoff closes

---

## Rejection

A receiving agent may reject a handoff if:

- Request is vague or multi-artefact.
- Acceptance criteria are not measurable.
- The handoff would violate a gate (dry-run default, scope non-goal, missing tests).

A rejection cites the reason and returns the handoff unchanged to the sender.
