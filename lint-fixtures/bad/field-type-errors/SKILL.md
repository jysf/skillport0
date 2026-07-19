---
name: 42
description: 42
compatibility: 123
metadata: flat-string-not-a-map
allowed-tools: 42
---

<!-- SPEC-012 coverage fixture: every open-spec field given the wrong YAML
     type at once (numbers/scalars instead of string/mapping/list-or-string).
     Fires: name.type, description.type, compatibility.type, metadata.type,
     allowed-tools.type (all warnings/errors per the catalog). -->

Body.
