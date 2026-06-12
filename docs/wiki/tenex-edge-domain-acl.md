---
title: Tenex-Edge Domain ACL
slug: tenex-edge-domain-acl
topic: tenex-edge
summary: "The domain has two verb planes: Project-State (open_project, roster, presence, status, project_meta, list_projects) and Communications (send, inbox, threads, th"
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-09
updated: 2026-06-09
verified: 2026-06-09
compiled-from: conversation
sources:
  - session:d208c058-7b2b-4ff8-bb82-d63623d51097
  - session:f3a730bf-9a3b-4952-b687-c93ade5fd7ec
---

# Tenex-Edge Domain ACL

## Domain ACL Structure

The domain has two verb planes: Project-State (open_project, roster, presence, status, project_meta, list_projects) and Communications (send, inbox, threads, thread_meta), with ACL (is_member) as a predicate both planes consult, not a third plane. The architecture requires a list_projects() enumeration verb in the Project-State plane (it was initially missing). The is_member ACL gate is consulted twice over the same store rows — once as a write-side admission predicate during materialization, and once as a read-side query — never on the wire. The domain ACL shows pending agents whose kind:0 p-tags the current user but haven't been authorized yet; the human decides allow or block. An injection hook surfaces these unauthorized agents to the human for decision.

<!-- citations: [^d208c-37] [^d208c-38] [^d208c-46] [^f3a73-115] [^d208c-50] -->
