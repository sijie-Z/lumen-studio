---
name: tracker
description: Project session lifecycle tracker. Use at the START and END of every coding session. Before any implementation, read CURRENT_STATE.md and TASK_BOARD.md via 'tracker status'. After completing work, call 'tracker finish' to record the session. This is mandatory in multi-agent environments where multiple CLI windows work on the same repo concurrently.
---

# Project Tracker

## Lifecycle (MANDATORY)

Every coding session follows this exact sequence. Skip no step.

### 1. START — Before any code changes

[Read-only: use python .project/tracker.py status to inspect current state.]

- Check which branch is active
- See what tasks are DOING (avoid stepping on another session)
- Read known issues
- If your task is in TODO, claim it: python .project/tracker.py start "<task-name>"

**Conflict detection**: if 	racker start warns the task is already DOING, warn the user before proceeding.

### 2. DURING — Record decisions as they happen

[Use python .project/tracker.py decide for architecture decisions.]

- python .project/tracker.py decide "JWT RS256" "More secure than HS256" "All services must validate with public key"
- python .project/tracker.py state file "path/to/changed.rs" — record changed files
- python .project/tracker.py state issue "description" — record known issues

### 3. END — Before closing the session

[Mandatory: run all of these before the user sees your final message.]

`
python .project/tracker.py state next_step "what to do next"
python .project/tracker.py finish "what was accomplished"
`

Then commit: git add .project/ && git commit -m "session: <summary>"

## Command Reference

| Command | When |
|---------|------|
| 	racker status | Session start — see current state |
| 	racker start "<task>" | Claim a task from TODO |
| 	racker todo "<task>" | Add new task to TODO |
| 	racker done "<task>" | Mark task complete |
| 	racker finish [summary] | Session end — log + update state |
| 	racker decide "<title>" "<reason>" "<impact>" | Record architecture decision |
| 	racker state file "<path>" | Record modified file |
| 	racker state issue "<text>" | Record known issue |
| 	racker state next_step "<text>" | Set next step |