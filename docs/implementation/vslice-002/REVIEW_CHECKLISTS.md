# Independent review checklists — VSLICE-002

Each review MUST be performed by an agent/person other than the primary implementation author, with a distinct context. Record reviewer, base commit, commands rerun, findings, severity, patches and retest. No review can be replaced by an implementation agent marking its own checklist complete.

## Review A — Qt, accessibility, UI trust

- [ ] Real semantic metadata produces layout without component-name-specific QML.
- [ ] No untrusted QML/JS evaluation, URL loading or property expression injection.
- [ ] GUI thread owns QObjects; worker thread executes guest/storage work.
- [ ] View teardown and room switch eliminate stale callbacks/use-after-free.
- [ ] Typed, bounded bridge honours generation/revision fences and backpressure.
- [ ] Keyboard traversal, labels, accessible errors and focus on result updates verified in Qt.
- [ ] Busy/pending/error handling preserves previously committed financial result meaning.
- [ ] Reviewed test instrumentation truly instantiates Qt components.

## Review B — Room persistence, integrity and registry boundaries

- [ ] Snapshot includes version, stable room/instance IDs, immutable exact component/UI pins, selected canvas and committed values only.
- [ ] Serialization cannot persist partial/uncommitted results as canonical outputs.
- [ ] Crash/corrupt/truncated data fail closed without deletion or unrequested overwrite.
- [ ] Safe temp-file/rename or SQLite transaction; path/symlink and size limits documented and tested.
- [ ] Exact pin is verified on restore; no silent fallback to latest version or new binary.
- [ ] Offline reopen does not require AI, registry network, or recompute solely to show committed data.
- [ ] Cross-room identity, forged UI edits and permission boundaries remain enforced.
- [ ] Two-process integration evidence is genuine, repeatable and isolated via temp dirs.
- [ ] Clear statement distinguishing clean-quit durability from full crash consistency.

## Review record template

```
Reviewer:
Role (A/B):
Implementation commit:
Review date:
Checks rerun:
Findings and severity:
Fix commit(s):
Retest commands/results:
Decision: APPROVE / REQUEST CHANGES
```
