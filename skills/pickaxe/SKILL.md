---
name: pickaxe
description: Investigate how any software works (a website, web app, desktop or phone app, game, program, file or API) with evidence, then plan your own version. Use when asked how something works, to reverse engineer, or to clone a feature from a behaviour spec.
---

# Pickaxe

Use the Pickaxe MCP tools to study software from the outside and keep an evidence-backed record.

1. Call `core_session_show` to see what is open and whether Clean room or Deep mode is active.
2. Call `core_investigation_show` for the brief, plan, hypotheses and findings so far.
3. Record anything you cannot yet prove with `core_unknown_add`, and settle it with `core_unknown_edit` once you have evidence.
4. Cite evidence ids in every answer and keep observations separate from inferences.

The hosted server at https://pickaxe.run/mcp is read-only and sandboxed. For deeper analysis install the local version from https://github.com/goldrusher9009-sketch/PickAxe.
