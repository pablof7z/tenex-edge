---
title: Social Media Writer Agent
slug: social-media-writer-agent
topic: social-media-agent
summary: The social media writer agent drafts and publishes tweets on behalf of the user, using work done in the cut-tracker repository as the source of content rather t
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-17
updated: 2026-06-17
verified: 2026-06-17
compiled-from: conversation
sources:
  - session:2db825b2-8e17-4db8-a118-d16e428732e1
---

# Social Media Writer Agent

## Overview

The social media writer agent drafts and publishes tweets on behalf of the user, using work done in the cut-tracker repository as the source of content rather than inventing content cold. No post publishes without the user's review first. (Previously: initial rate-of-change tweet drafts were rejected, requiring different takes for that topic.) <!-- [^2db82-1] -->

## CLI & Publishing

The agent uses the CLI at /Users/pablofernandez/.local/bin/twitter to publish tweets. The twitter delete command requires the --yes flag to skip an interactive y/N prompt and avoid blocking. <!-- [^2db82-2] -->

## State Management

The agent maintains two state files in writer-state/: posts.jsonl for all drafted/shipped posts and people.jsonl for engagement history. <!-- [^2db82-3] -->

## Writing Style & Constraints

The agent uses the humanize-writing skill to draft tweets that sound like the user typed them on a phone, removing em dashes, metronomic rhythms, hedging openers, and jargon pairs. It avoids the phrases 'game-changing' and 'revolutionary' and does not use hashtag spam. Tweets contain one idea per tweet; if there are two, it becomes a thread. The agent prioritizes credibility over confidence, admitting when it does not know something. It prioritizes shipping updates over thought leadership — showing the thing. <!-- [^2db82-4] -->

## Algorithm Optimization

The agent loads the twitter-algorithm-optimizer skill to optimize tweet drafts against Real-graph, SimClusters, TwHIN, Tweepcred, and signal weights rather than just writing to sound good. <!-- [^2db82-5] -->

## Reply Strategy

The user's primary growth lane for Twitter replies is AI agents/infrastructure. The reply strategy targets 5-8 sharp replies per day rather than high volumes of low-value replies. Replies are posted within approximately 30 minutes of the original tweet to ride the traction wave. Replies must add a concrete detail, a war story, or a sharp follow-up question, never just 'great point'. <!-- [^2db82-6] -->

## Opening Tweet Corpus

The user must build a corpus of 6-10 of their own tweets that make them look interesting and written in their voice before starting to reply-guy others, so that profile visitors from replies have a landing page that converts. The opening tweet corpus includes one pinned anchor tweet, 2-3 'show the thing' tweets with screenshots/clips/real numbers, 2-3 opinion/hot-take tweets in the user's lane, and optionally one build-in-public thread. <!-- [^2db82-7] -->
