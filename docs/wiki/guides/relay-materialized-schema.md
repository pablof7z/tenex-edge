---
title: Relay-Materialized Database Schema
slug: relay-materialized-schema
topic: relay-materialization
summary: The database must contain only relay-materialized state, with no local tables or columns that deviate from relay events.
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-28
updated: 2026-06-28
verified: 2026-06-28
compiled-from: conversation
sources:
  - session:b9176726-a9a8-41a9-b806-c966e8c94ed7
---

# Relay-Materialized Database Schema

## Source of Truth

The database must contain only relay-materialized state, with no local tables or columns that deviate from relay events. <!-- [^b9176-c6860] -->

## Entity Property Queries

Entity property queries (such as admin status) must be answered from relay-materialized state, not from local creation flags like `owns_group`. <!-- [^b9176-c6860] -->

## Schema Changes

The `owned_groups` table is deleted entirely. <!-- [^b9176-7d79d] -->
