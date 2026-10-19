---
id: ai-and-privacy
title: AI Assistant and Privacy
anchors: privacy, access-levels, redaction, keychain
phase: 0/5
---

The AI assistant is optional. You can do every task in CatDot Studio manually or through Guided Mode with no AI provider configured. This article is about what the AI features do, what actually leaves your machine, and how your credentials and workspace are protected.

## AI is optional

The **AI assistant** page streams chat over an OpenAI-compatible provider you configure in Settings (base URL, model, API key). Without configuration it says "Configure an AI provider in Settings first" and does nothing else - there is no background traffic. Manual Mode and Guided Mode call the same local tools as any AI-driven flow would, so skipping AI costs you no capability in today's release. The autonomous AI agent loop that will plan and execute multi-step tasks is **planned (Phase 5)** and not yet available.

## What leaves your machine

Today, the only network operation is chat: **exactly the message you type** is sent to your configured base URL and streamed back. Your APK contents are not attached automatically. The policy engine treats content-bearing network requests as needing explicit consent - pressing **Send** is that consent - and the audit trail stores only a SHA-256 hash and byte size of a payload, never its content. There is no telemetry by default; no APK data is ever uploaded by the app itself.

## Access levels and approvals

Settings -> **Safety access level** persists one of six levels, enforced in Rust (not by prompt text) with a tested policy table:

- **L0 Chat only** - no project tools at all.
- **L1 Read selected** - reads only files you attached or approved.
- **L2 Read workspace** - read-only over the project.
- **L3 Edit with approval** - edits, builds, and device actions need your approval.
- **L4 Auto edit** - edits run automatically (with checkpoints); builds and device actions still confirm.
- **L5 Full pipeline** - everything except destructive actions, which always require explicit confirmation.

Network sends require consent at *every* level. The agent that will exercise these levels against tool calls is planned for Phase 5; the policy engine, level persistence, and consent gates are real today.

## Redaction and credential storage

- **Redaction:** a filter scrubs things that look like secrets (API keys, bearer tokens, passwords, long high-entropy strings) out of logs, audit entries, and diagnostic output, and normalizes your home folder path to `~` so usernames never leak.
- **Keychain:** your API key is stored in **Windows Credential Manager** via the OS credential store, not in the settings database. Settings only shows "A key is saved for this account" - the key itself is never echoed back into the UI or logs.
- **Local by design:** projects, patches, and checkpoints stay on disk under the app data folder.

If you want AI help but dislike sending text to a cloud provider, a local model endpoint (Ollama/LM Studio style) works with the same OpenAI-compatible settings; it is still your choice to press Send.

Related: [getting-started](getting-started.md), [checkpoints](checkpoints.md), [toolchain-setup](toolchain-setup.md)
