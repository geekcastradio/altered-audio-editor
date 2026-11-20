# Hand test 0.47.0: backups and templates still work

1. Edit > Settings > Backup and Restore: the backup folder and the list of snapshots show. Press Back Up Now: a new snapshot appears in the list and the file name carries the app version.
2. Export Settings Backup to a file, change a setting, then restore from that file. The setting comes back. Restoring first makes a "before restore" snapshot.
3. Change the number of snapshots to keep. Close and start the program: the number is kept.
4. File > New from Template: the built-in starters list. File > Save as Session Template: save the open session, it shows in the list. Delete it: it goes.
5. Portable copy (if you use one): the same checks, with files in `data\config` and `data\config\templates`.
6. Nothing else should look or behave differently.
