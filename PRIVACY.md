# Privacy Policy

nia does not collect or transmit personal data. Documents and settings stay on your computer.

nia writes brief operational logs to the process's standard error stream, such as document open and save events, theme changes, and failures. Logs include the source module and, for file errors, the error kind. Set `NIA_LOG=debug` when you launch nia to include additional editor state changes, such as search scope and autosave scheduling. Logs do not include document text, search queries, or file paths. nia does not transmit logs. If you launch nia from a terminal or another program that records standard error, that program may retain the logs.

For crash recovery, nia keeps temporary local copies of unsaved document text in its application support directory. It removes a draft after that document is saved or its clean tab is closed. Recovered drafts appear in separate groups when nia starts again. The recent-files list and saved tab group layouts store local file paths in the same application support directory. The selected font family name is also stored there.
