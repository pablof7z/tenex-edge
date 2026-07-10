---
title: Chief-of-Staff Workflows
slug: chief-of-staff-workflows
topic: agent-workflows
summary: The chief-of-staff agent manages workflows through the `scripts/workflows.py` script, which is the existing mechanism used to interact with and manage chief-of-
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-07-10
updated: 2026-07-10
verified: 2026-07-10
compiled-from: conversation
sources:
  - session:4d78f854-d5e5-4a11-b0d4-358f33111d15
---

# Chief-of-Staff Workflows

## Overview

The chief-of-staff agent manages workflows through the `scripts/workflows.py` script, which is the existing mechanism used to interact with and manage chief-of-staff workflows. Workflows belong in a tracking repo rather than a bare home directory. Once a tracking repo exists, the `workflows/` directory should be moved into it (e.g. `<tracking-repo>/chief-of-staff/workflows/`) and `~/.agents/homes/chief-of-staff/workflows` replaced with a symlink into the clone. This should be done the first time there is both a workflow worth keeping and a tracking repo to put it in, without waiting to be asked. <!-- [^4d78f-82563] -->

## Symlink Enforcement

The `scripts/workflows.py` script runs a symlink check on every invocation and emits a loud stderr warning (`⚠️ workflows/ is a plain directory, not a symlink into a git repo...`) whenever `workflows/` exists but is not a symlink, staying silent once it is a symlink. <!-- [^4d78f-c2780] -->

## Available Workflows

The chief-of-staff agent maintains six available workflows: bug report triage, feature request triage, decisions kanban, ops board, inbox monitor, and knowledge capture. <!-- [^4d78f-f7685] -->

## Bug Report Triage

The bug report triage workflow takes an informal bug flag (symptom/project, not repo/file), identifies the right repo, checks tenex-edge for prior context, searches existing issues to avoid dupes, then files an evidence-backed GitHub issue plus a durable record in `everything/investigations/` (and a decision entry if judgment is needed). <!-- [^4d78f-35497] -->

## Feature Request Triage

The feature request triage workflow takes a loose feature description (example commands/JSON, not a literal spec), grounds it in the actual repo code first (not just docs/commits), then files a precise issue on the owning repo plus a note in `projects/<slug>/`. The agent does not implement unless asked. <!-- [^4d78f-964d3] -->

## Decisions Kanban

The decisions kanban workflow places items needing the user's judgment on Project 5's 'Needs Pablo' column as GitHub issues in `pablof7z/everything`. The user comments to answer and the agent acts on it and closes/moves it, with the user only watching that one column. <!-- [^4d78f-bc913] -->

## Ops Board

The ops board workflow provides broader in-progress-work visibility via Project 5 with columns for Status (Pablo's Inbox / In Progress / Blocked / Done), Type, Owner, and Repository. The user only watches the 'Pablo's Inbox' view while everything else is passively maintained by the agent. <!-- [^4d78f-d4505] -->

## Inbox Monitor

The inbox monitor workflow is a standing launchd background loop (`inbox-watch-loop.sh`) polling Project 5's 'Pablo's Inbox' column every ~8s, detecting new comments from the user, and dispatching a headless Claude session to act on them automatically. <!-- [^4d78f-afba2] -->

## Knowledge Capture

The knowledge capture workflow files handed-over material (launch docs, positioning, domain knowledge) as `projects/<slug>/kb/<category>/<entry>.md` in the `everything` repo, additively and same-session, not left to live only in chat. <!-- [^4d78f-c5ff9] -->

## Unknown Tasks

Tasks that don't fit any of the six defined workflows are treated as `unknown-task`: the agent does them directly if simple/reversible, asks only if the action changes priority/authority/money/irreversible state, and captures a new workflow once the shape is clear. <!-- [^4d78f-dea36] -->
