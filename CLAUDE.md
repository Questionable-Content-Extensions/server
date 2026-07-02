# Prelude

This CLAUDE.md is shared between qcext-server and qcext-client. The latter does not have a database or a backend, so when operating in qcext-client, those sections can be ignored.

# Critical Rules (read first)

- **Never read `.env`**.
- **Never run `git commit`, `git push`, `git stash`** or other git commands that cause repo or index mutation unless the user explicitly says to.
- **Never delete unused code** — mark it as unused instead, or ask.
- **Run end-of-feature checks** (see Frontend/Backend sections) before declaring work done.
- **Never manually edit `bindings/*`.**
- **Never pipe `./run_*` script output through `head` or `tail`** — if you need to filter or reduce output, pipe through `jq` instead, e.g. `./run_eslint.sh | jq .count`.

# General

Use the Read tool to read file contents. Do not use shell commands (`cat`, `head`, `tail`, `sed`, `awk`, etc.) to read files. Shell commands may be used to _find_ and _filter_ things in files.

# Database

To add a new migration to the database, cwd to `./database` and run `cargo sqlx migrate add <DESCRIPTION>`. The file created is not empty, but contains a comment, so the Read tool must be used before the Edit tool or Write tool will succeed writing to the new file. Once the migration file is finished, run `cargo sqlx migrate run` to update the database.

**Never manually edit `./database/.sqlx/*`**.
**Never manually run `cargo sqlx prepare`**.

You can use the database url `mysql://claude:claude@127.0.0.1:3306/dphoenix_qcnav` for db access. It provides read-only access.

# Commits

Commit messages must follow Conventional Commit formats. These should be single-line and under 100 characters long. This is enforced with a git hook. Do not ever manually run `git commit` unless explicitly instructed to by the user.

# Code changes

Never delete code just because it appears unused — not even when refactoring. Ask the user first. Suppressing the warning (`#[expect(dead_code)]`, `_` prefix, etc.) is always acceptable instead.

Compiler/lint errors during a multi-step edit are expected and should be ignored. Only run the end-of-feature checks (see below) once all planned changes are complete.

Every code change must be accompanied by one or more tests. If the code being modified has no existing test coverage, write a passing test that captures the current behavior _before_ making any changes — this acts as a regression guard. Then make the change and add or update tests to cover the new behavior.

## Frontend

Don't use inline/naked type definitions like `{ field: string }`, always create a named type using `interface` (preferred) or `type` (when necessary) somewhere related/logical and use it instead. If the type also exists on the backend, use `ts-rs`'s export system to create a binding for it instead of redefining it manually.

Avoid `as any` and `as unknown` type assertions, and avoid typing variables or parameters as `any` or `unknown`. Only do so when the correct type is genuinely impossible to express or would require disproportionately large effort — and in those cases, add an inline comment explaining why.

Once a frontend feature is finished, run these checks in order:

1. `npx tsc --noEmit` — TypeScript errors
2. `./run_eslint.sh` — ESLint violations. To check only a specific file/directory, run `./run_eslint.sh <path>`.
3. `./run_react_tests.sh` — React unit test suite
4. `./run_storybook_tests.sh` — Storybook test suite

The tests can be skipped if program logic obviously didn't change, like editing on-page text or extracting an explicit type declaration from an inline-declared type in Typescript.

The lint script outputs JSON in this format:

```json
{
    "count": 1, // total count of lints
    "lints": [
        {
            "file": "",
            "messages": [
                {
                    "ruleId": "",
                    "severity": 0,
                    "message": "",
                    "line": 0,
                    "column": 0,
                    "messageId": "",
                    "endLine": 0,
                    "endColumn": 0
                }
            ]
        }
    ]
}
```

Both test scripts output JSON in this format. **`results` contains only failing files/tests** — files where all tests pass are omitted entirely, and within each file only failed assertions appear. **`noisyPasses` contains only passing files/tests that produced log output** — files/tests with no logs are omitted entirely. Both `results` and `noisyPasses` should be inspected by the agent for issues; as a passing test may still have lesser issues that need addressing but which are only surfaced in console output.

```json
{
    "tests": 0, // total count across all test files
    "passed": 0, // total passing
    "failed": 0, // total failing
    "results": [
        // only files with at least one failure
        {
            "file": "",
            "errors": [] // test scaffolding errors
            "results": [
                // only failed assertions within this file
                {
                    "name": "", // full path to test file
                    "component": "", // Storybook only
                    "errors": [], // test assertion errors
                    "logs": [{ "type": "stdout | stderr", "content": "" }] // console output logged during this test
                }
            ]
        }
    ],
    "noisyPasses": [
        // only files with at least one passing test that produced logs
        {
            "file": "",
            "results": [
                // only passing assertions within this file that produced logs
                {
                    "name": "", // full path to test file
                    "logs": [{ "type": "stdout | stderr", "content": "" }] // console output logged during this test
                }
            ]
        }
    ],
    "unassociatedLogs": [{ "type": "stdout | stderr", "content": "" }] // console output that couldn't be tied to a specific test
}
```

Errors have this shape:

```json
{
    "stack": "",
    "message": "",
    "cause": {
        "stack": "",
        "message": "",
        "constructor": "",
        "name": "",
        "toString": ""
    },
    "constructor": "",
    "name": "",
    "toString": "",
    "stacks": [
        {
            "line": 0,
            "column": 0,
            "file": "",
            "method": ""
        }
    ]
}
```

### Storybook

**For a single component, always use:**

```sh
./run_storybook_tests.sh <ComponentName>.stories
./run_react_tests.sh <ComponentName>.test
```

Never run the full suite just to check one component's output — it wastes CPU cycles and adds significant RAM pressure. Conversely, running the full suite covers all tests, so there's no reason to run single tests in addition to the full suite.

Changes that introduce new UI should have a corresponding Storybook story made about them, so that visually inspecting them is easy.

### Redux / RTK Query

When adding or removing a Redux slice (or an RTK Query `createApi` instance), review `rootReducer` in [src/client/redux/store.ts](src/client/redux/store.ts). It deliberately resets certain slices on logout and auth expiry to prevent data leakage between users — a new slice that holds per-user data must be added to that reset block, and a removed slice must be dropped from it.

## Backend

When adding a crate, always check what the latest version is with `cargo search <crate-name> --limit 1`, never assume you know what the latest version is. Unless the user explicitly says otherwise, always use the latest version of a crate.

All SQL queries (`sqlx::query`, `sqlx::query_as`, `sqlx::query_scalar`, etc.) belong in the `./database` crate — never directly in main-crate models or controllers.

Once a backend feature is finished, run these checks in order:

1. `cargo clippy --workspace --all-targets` — Clippy/compiler errors
2. `cargo test --workspace -- --skip export_bindings --skip __export_endpoint` — tests

If any type that affects bindings is added/modified/deleted, run `./update_bindings.sh` so that `ts-rs` can reproduce the bindings for the frontend (written to `./bindings/`). _Do not manually edit files within `./bindings`_.

Organize functions top-down: higher-level concepts and callers come first, helpers and lower-level functions come later. A function should never call a function defined above it in the file.
