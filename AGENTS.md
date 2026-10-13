# Repository instructions

## Commit workflow

When the user says "commit", commit the requested changes locally and push them to the current branch's configured upstream. The request explicitly authorizes both actions, including pushing to `origin/main` when working on `main`. Do not ask for separate push confirmation.

Verify the working tree and upstream state afterward. Report "committed" only when the commit is pushed and synced. If either action is blocked or fails, state exactly what remains incomplete.

## New chat handoff

When the user says "time for a new chat", prepare a concise, complete handoff and create a new local chat. Keep the current chat's title prefix and increment its trailing number by one; for example, `dLive-REAPER-Link 8` becomes `dLive-REAPER-Link 9`. Check existing chats to avoid creating a duplicate.

Put the handoff in the new chat's opening prompt. Include the repository path, current version and Git state, completed work and verification limits, agreed decisions, remaining work, and whether the next chat is planning or implementation. Read these repository instructions in the new chat and preserve the user's authorized scope. Do not start implementation when the user has requested planning only. A separate handoff file is unnecessary unless requested.

Prefer the same project and a local execution environment. If the app cannot create a local chat directly inside the ChatGPT project, the user authorizes creating it as a local projectless chat with the same handoff. Associate it with the original project afterward when supported; otherwise report that it remains outside the project. Do not switch to a cloud chat as a workaround.
