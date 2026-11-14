# Hand test: six more dialogs on the shared shell (0.27.0)

For each dialog check: it opens with the right control focused, Escape closes it, a click outside closes it, a drag that starts inside and ends outside does not, and Cancel is on the left with the main action last.

1. Help > About. Logo, version and the privacy note show. OK closes it. Text can be selected and copied.
2. Help > Check for Updates (when an update exists, or with a test build). The title reads "Version X is available". Later, Release Page and Install and Restart sit in that order. While installing, Escape and a click outside do nothing.
3. The welcome pages (delete the welcome-seen setting, or use a fresh profile). Next and Back step through pages. "Open the User Guide" sits on the left of the footer. The last page has Start Editing or Got It.
4. Help > User Guide. The search box is focused. Typing narrows the section list. Picking a section shows its text and pictures. The reader scrolls inside the dialog and the dialog stays within the window at a small size. Close works.
5. File > New Session. Name is focused and selected. Enter in the name or folder field confirms, Enter in a drop-down does not. Browse works. Picking a template locks rate, depth and mix and shows the template note. The trash button deletes a template after its confirmation. Cancel and Escape change nothing.
6. File > Session Properties. Name is focused. Changing the rate shows the note about converted files. Pan law and loudness target change their notes. OK applies, Cancel and Escape do not.
