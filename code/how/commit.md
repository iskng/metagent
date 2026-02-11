# GIT COMMIT INSTRUCTIONS
- The user requests that if there have been any changes to the code you must commit them. 
- But only commit when there are actual code changes. If only @plan.md or session notes changed, skip committing.
- Subject format: `feat({task}): <concise summary> (plan: 1,2,3)`
- Body format: bullets describing major changes, wrapped at ~72 columns.
- Each bullet should mention the key file/class/module it affects.
- Only stage and commit files relevant to the task; ignore unrelated changes.
- If there are no code changes (e.g., only @plan.md/session notes updated), skip git add/commit for this loop.
- Only commit if there are actual code changes.

Example:
feat(ios): add fluid input conditioning and refactor local reranking (plan: 1,3,4)

- Implement `InputConditioner` using vDSP to apply RMS-based gain
  normalization in `FluidTranscriber` for better low-volume capture.
- Add a toggle for "Fluid Input Conditioning" in the iOS Settings
  screen and update `SettingsViewModel` to handle the new state.
- Refactor `lib_actors` by removing `llama_rank.rs` and migrating
  ranking logic into `backend_local.rs`.
