---
name: simplified-technical-english
description: Use before ANY prose this repo emits, in every session and every subagent. This INCLUDES every message an agent prints to the console: the reply to the operator, progress narration, and a subagent's final report to its caller. No console message is exempt for being short, informal, or interim. Prose means chat replies, CLAUDE.md and docs, code comments and docstrings, commit messages, PR titles and bodies, review findings, error and log strings a person reads, plans, and status reports. ASD-STE100 Simplified Technical English is the house writing standard. Invoke when about to author or edit a markdown file, write a commit message, open a PR, report status, or reply at length. Do NOT invoke to rewrite code identifiers, quoted source text, command output, or protocol-controlled strings.
allowed-tools: Read Grep Bash(grep:*) Bash(sed:*)
verified: 2026-09-20
verified-against:
  - CLAUDE.md
  - .claude/skills/add-claude-md/SKILL.md
review-cadence: on-architectural-change
---

# Simplified Technical English (ASD-STE100)

## Purpose

ASD-STE100 is a controlled-English specification. It limits vocabulary, sentence length, and grammar so that a reader with different first languages, and a machine reader, get one meaning from one sentence. This repo adopts it as the house standard for all prose that agents write.

The standard applies to every agent, every subagent, and every skill in this repo. An agent that writes prose applies these rules without a separate instruction.

Console output is prose. The operator reads the terminal, so the terminal is the primary surface for this standard, not an exception to it. An agent holds the same bar for a one-line progress note as for a committed document.

## Scope: what counts as prose

Apply the rules to:

- EVERY message an agent prints to the console. This covers the reply to the operator, progress narration between tool calls, the closing summary of a turn, and the final report a subagent returns to its caller. A short message is still a message. An interim message is still a message.
- Status reports, findings, and hand-off notes between agents.
- CLAUDE.md files, README files, ADRs, runbooks, and roadmap documents.
- Code comments and docstrings.
- Commit messages, PR titles, and PR bodies.
- Review findings and audit reports.
- Error strings and log strings that a person reads.
- Plans and task lists.

## When NOT to use

Do not apply the rules to these. Reproduce them exactly:

- Code identifiers, type names, API names, file paths, and command names.
- Quoted text from another source. Keep the quotation exact.
- Command output, stack traces, test output, and diagnostic text from a tool.
- Strings that a specification, a protocol, or a wire contract controls.
- Third-party content that this repo copies rather than authors.

Do not read the skill file again for a pure code edit that adds no prose to the diff. Do not invoke it to relitigate wording that a lint rule already fixed.

These are exemptions for CONTENT, never for a surface. A turn that edits only code still ends in a console message, and that message follows the standard.

## Word rules

1. One word carries one meaning. Do not use one word as both a noun and a verb. Use "the test" as the noun. Use "examine" as the verb.
2. One term names one thing. A rename mid-document creates a second thing in the reader's model.
3. Use the simplest correct word. Prefer "use" over "utilize". Prefer "start" over "initiate". Prefer "before" over "prior to".
4. Group three nouns at most. Write "the timeout for the request handler", not "the request handler timeout value".
5. Do not use slang, jargon, or idiom. Ban phrases such as "under the hood", "out of the box", and "low-hanging fruit".
6. Expand an abbreviation at its first use, then give the short form in parentheses.
7. Keep the articles "a", "an", and "the". Keep "that" and "which". These words show the structure of the sentence.

## Sentence rules

1. Keep an instruction to 20 words or less.
2. Keep a descriptive sentence to 25 words or less.
3. Write one instruction in one sentence. Two steps need two sentences.
4. Use the active voice. Write "The server sends the token", not "The token is sent by the server".
5. Use the simple present, the simple past, or the simple future.
6. Do not use the -ing form as a verb or as a noun. Write "This function reads the file". Write "The build failed because the type is wrong".
7. Write the condition first, then the instruction. Write "If the test fails, read the log".

## Structure rules

1. Keep a paragraph to six sentences or less.
2. Put one topic in one paragraph.
3. Use a vertical list for three or more items, or for a sequence of steps.
4. Use a numbered list for ordered steps. Use a bulleted list for unordered items.
5. Put a warning or a caution before the instruction that it applies to.

## Tone rules

1. Give an instruction as a command. Write "Run the migration".
2. State a fact directly. Delete "basically", "essentially", and "just".
3. Say what to do, not only what not to do.
4. Do not hedge a verified result. Report a measured outcome in plain words.

## Approved substitutions

| Do not write | Write |
|---|---|
| utilize, leverage | use |
| initiate, kick off | start |
| terminate | stop, end |
| prior to, in advance of | before |
| subsequent to, following | after |
| in order to | to |
| due to the fact that | because |
| at this point in time | now |
| a number of, several | the count, or the exact number |
| attempt to | try |
| facilitate, enable | let, allow |
| it is worth noting that | (delete the phrase) |
| potentially, possibly | (state the condition that causes it) |

## Rewrite examples

| Before | After |
|---|---|
| It's worth noting that the config is basically just being read at startup. | The application reads the config at startup. |
| Prior to utilizing the API, authentication needs to be performed. | Authenticate before use of the API. |
| The request handler timeout configuration value | The timeout for the request handler |
| This might potentially cause issues down the line. | This causes an error later. |
| Running the tests will validate the changes. | Run the tests. The tests show if the changes are correct. |
| Failed parsing input file | The parser did not read the input file. |
| We went ahead and refactored the service layer. | This change refactors the service layer. |

## Self-check before send

Run this check on drafted prose before a console reply, a commit, or a PR body:

1. Count the words in the longest sentence. An instruction over 20 words needs a split.
2. Search for "-ing" used as a verb or a noun. Replace each one.
3. Search for the passive voice. Name the actor and make it the subject.
4. Search the approved-substitutions table's left column. Replace each hit.
5. Count the nouns in the longest noun group. Three is the ceiling.
6. Confirm that each term names the same thing throughout the document.
7. Confirm that a condition precedes its instruction.
8. Read the console reply once more. The reply is the surface the operator reads first.

A fast mechanical pass over a drafted markdown file:

```bash
grep -nEi "utiliz|leverag|prior to|in order to|worth noting|basically" FILE
```

## Interaction with repo lint

**NO MACHINE CHECKS THIS STANDARD. Every rule here holds by review.** No linter in this repository reads a CLAUDE.md file, a SKILL.md file, or an agent definition for register, length, duplication, or cross-references. The native git hook (`scripts/dod.sh`) checks code, not prose.

Earlier versions of this section said the linter enforced a narrow subset: no em dash or double hyphen between words, no second-person address, no motivational prose, and no emoji. Two corrections followed, and the second one removed the mechanism entirely. First, those checks never reached an agent definition under `.claude/agents/`, which took a narrower path; the claim that they did was false, and it taught every agent that invoked this skill to expect coverage that did not exist. Then the whole linter went away.

So the sentence that was once a caveat is now the entire truth: a green run was never proof of compliance, and today there is no run at all. Hold the standard because it is the standard.

The house conventions for CLAUDE.md files live in the `add-claude-md` skill (the decision tree, Shape 1, and the frontmatter list), and the `claude-md-audit` skill is the periodic sweep. Read them for intent, and expect no command to check the result.
