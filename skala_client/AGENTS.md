# Client Guidelines

## Scope

These instructions apply to `skala_client/**`.

## Source And Generated Files

- YueScript files (`*.yue`) are the client source. Generated Lua files (`*.lua`)
  and `bin/` output are build artefacts.
- Do not edit generated Lua as source. Change the corresponding `.yue` file and
  rebuild.
- `skala/server_types.yue` is generated from Rust quicktype-derived server API
  types, but is committed to keep the client build simple.

## Coding Standards

- Follow the existing YueScript style, import layout, and module organisation.
- New functions must include explicit quicktype annotations, usually with
  `$F 'signature', (...) ->`.
- Use `declare_type` for supporting quicktype shapes when a function signature
  needs a named structure or union.
- If more detail is needed about the quicktype notation used by this project,
  consult `https://kcza.net/shunt/reference-materials/quicktype-notation/`.

## Verification

- Use `make -C skala_client` from the repository root to compile YueScript to
  Lua.
- If a client change depends on server API type changes, also follow the server
  quicktype verification guidance.
