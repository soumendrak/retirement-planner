---
description: Keep AGENTS.md updated with project context and decisions
---

# Update AGENTS.md Workflow

This workflow ensures the AGENTS.md file stays current as the project evolves.

## When to use

- At the start of each conversation about this project, read AGENTS.md to understand context
- After making significant decisions or completing milestones, update AGENTS.md
- When implementation phases progress, update the status column

## Steps

1. **Read AGENTS.md** at the start of a new conversation to understand:
   - Project constraints and decided requirements
   - Current implementation phase
   - Data model and architecture decisions
   - Recent changelog entries

2. **After meaningful changes**, update the relevant sections:
   - **Decided Requirements** — add new decisions
   - **Implementation Phases** — update status (Pending → In Progress → Done)
   - **Conversation Changelog** — add a dated entry summarizing what was discussed/decided/done

3. **Changelog format** — use this pattern:
   ```
   ### YYYY-MM-DD (Short Title)
   - Bullet point of decision or change
   - Another bullet if needed
   ```

4. **Keep it concise** — AGENTS.md is a reference, not a full spec. Link to other docs if details grow too long.

## Example updates

- Phase 1 moves from "Pending" to "In Progress" when you start the Cargo workspace setup
- Add a new decision to the Decided Requirements table if the user chooses a different approach
- Log a changelog entry when a major feature is completed
