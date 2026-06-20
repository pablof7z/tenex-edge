---
title: TestFlight Deploy Workflow
slug: ci-testflight-deploy
topic: ci-cd
summary: The TestFlight deploy workflow triggers only when the iOS app version number (CFBundleShortVersionString in App/Resources/Info.plist) increases, or on manual di
tags:
  - capture
volatility: warm
confidence: medium
created: 2026-06-13
updated: 2026-06-13
verified: 2026-06-13
compiled-from: conversation
sources:
  - session:74fce09f-02b4-496f-a5e1-52d19ef9fbcd
---

# TestFlight Deploy Workflow

## TestFlight Deployment

The TestFlight deploy workflow triggers only when the iOS app version number (CFBundleShortVersionString in App/Resources/Info.plist) increases, or on manual dispatch, rather than on every push to main. To ship a TestFlight build, bump the version in App/Resources/Info.plist (e.g. 1.0.0 → 1.0.1) and push, or trigger manually via 'gh workflow run testflight.yml'. <!-- [^74fce-6] -->

The TestFlight deploy gates on the unit test suite only (SKIP_UI_TESTS=1), while the full UI suite continues running in the regular Test workflow for coverage. <!-- [^74fce-7] -->
