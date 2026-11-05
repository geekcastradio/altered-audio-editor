# Altered Audio Editor 0.17.0

Libraries you can close, find and carry to another computer, a color and cover popup that stays next to the badge, and settings in one clear folder.

## Libraries

- **Closed libraries stay on your list.** Close Library now just hides it. Open the library list under the show name and click a closed one to bring it back. No more hunting for the folder.
- **Remove Library from List.** Right-click and choose it to take a library off your list. Nothing is deleted: the folder and its files stay where they are, and the confirmation tells you where.
- **Find Libraries in Folder.** Pick a folder and the app adds every library it finds in it and in the folders directly inside it. Open Library does the same when you pick a folder that holds libraries instead of being one.
- **Missing folders are handled.** If a library's folder has moved, click it in the list and choose Locate Folder, or take it off the list.
- **Export and Import Library List.** One small file holds the names, colors and folders of your libraries. Import it on another computer to rebuild the list there. The file does not carry the audio, so copy the library folders across as well. Importing never overwrites anything.

## Show badge

- **Show Color and Cover is one popup now.** Click the badge and the popup opens right under it, with the color swatches, Other color and Automatic, plus the cover: choose a picture from the library, choose a file, or remove it.
- **Drop a picture on it.** Drop a JPEG or PNG on the popup and it is copied into the library and set as the cover. Anything else is turned away without importing it.

## Settings and files

- **Settings moved to `Altered Audio`.** They now live in `%APPDATA%\Altered Audio` instead of a folder named after the app identifier. The first launch copies your existing settings across, checks every copy, and only then removes the old files. Nothing already in the new folder is overwritten. Portable mode is unchanged.
- **Backups and Cache moved next to your libraries.** They now live in `Documents\Altered Audio`, with Libraries and Sessions, instead of a second folder called `Altered Audio Editor`. Existing folders are renamed into place, so even a large cache moves at once. If you picked your own cache folder in Settings, it is left alone.

## Under the hood

- Files dragged in from the file manager now go to the smallest panel under the pointer, so a popup on top of a panel can take a drop of its own.
- More tests for the library list, the cover popup, the drop routing and the settings move.
