# nia handbook

nia is a Markdown editor. It saves text in `.md` files.

nia has a copy of this handbook. You can read it without an Internet connection. Select **Handbook** to open it. Select **Back to writing** to close it. You can also press Escape.

## Start a document

- Press Primary+N or Primary+T to make a document in a new group. See **Use keyboard shortcuts** for the Primary key.
- Select **Open** to open a `.md` file.
- Drop a `.md` file on the window to open it.
- Use `nia notes.md` at a command line to open a file when nia starts.
- Select **Save** to save a new document to a file.
- Select **Save As** to save the document to a different file.

After a short pause, nia saves changes to a named file. The footer shows the save status and the number of words.

## Use tabs and panes

nia starts with one tab group and one writing pane. Press Primary+T to make a new tab group with one blank document. Each group has its own pane layout. The small tab bar appears when you have more than one group. Select a tab in the bar or press Primary+1 to Primary+9 to change groups.

Press Primary+W to close the active document. If it is the last document in its group, nia closes the group. If you close the last view of a document with unsaved changes, nia asks you to save, discard, or cancel. nia also asks this question when you quit.

Press Ctrl+Shift+L to show one pane in the current group. Press Primary+D to add a new document in a pane on the right. Press Primary+Shift+D to add a new document in a pane below. Press Primary+] or Primary+[ to move between panes in the current group.

Select a pane to work in its document. Press Primary+N to make a new document group. Opening a different file makes a group when another document is open. To show the same file in two panes, open it again. If the active pane is empty, nia puts the file in that pane. If it is not empty, nia adds a pane on the right. Changes in one pane appear in the other pane.

Drag a pane by its title bar to move it. Drag the line between panes to change their sizes. Select the close button on a pane to hide that pane. This action keeps its document open. nia restores the groups, open documents, and pane layouts when you start it again.

## Write and read Markdown

Type Markdown in the writing area. For example, type `# Heading`, `**bold**`, or `- item`. The editor uses color to show headings, links, lists, and other Markdown syntax. The syntax stays visible as you write.

Select text and press Primary+B to add bold markers or Primary+I to add italic markers. If no text is selected, nia adds the markers and puts the cursor between them. Select **Insert** in the footer to add a table, blockquote, ordered list, code, fenced code block, link, horizontal rule, footnote, definition list, strikethrough, task list, emoji code, highlight, subscript, or superscript. The menu writes Markdown source text.

Type an opening parenthesis, bracket, brace, angle bracket, quote, apostrophe, or backtick to add its closing character. If you select text first, nia puts the pair around it. Type the closing character to move past it. Press Backspace between an empty pair to remove both characters.

Press Primary+Z to undo. Press Primary+Shift+Z to redo. nia groups nearby letters into word-sized undo steps. A space or punctuation ends the group. Pausing or moving the cursor also starts a new step. Pasting or adding Markdown with **Insert** is one step. Each document keeps its most recent 1,000 undo steps until you close it.

Read mode shows common Markdown forms. Some extended forms, such as definition lists, emoji codes, highlights, subscripts, and superscripts, stay as source text in Read mode.

Select **Read** to see formatted text. Select **Write** to return to the same text and cursor position.

To show an image while you write, put its Markdown link on a line by itself. For example, type `![Chart](images/chart.png)`. Move the cursor to another line to see the image. Select the image to edit its Markdown link. Save a new document before you use a relative image path. nia finds a relative image path from the document's folder. It does not load images from the Internet.

Read mode also shows local images. In Read mode, select a link to copy its address.

## Find text and headings

Select **Find** to search the active document. The search field opens above the writing area. Select **All tabs** to search all open documents, including documents in other groups and changes that you did not save.

Select a search result to show its document and select the text. Press Enter to go to the next result. You can also use the up and down buttons. Select **Case** to match uppercase and lowercase letters exactly. Press Escape once to close the search field.

Select **Outline** to see the headings in the active document. The outline stays open when you select a different tab. Type in the outline field to show only headings that match your text. Select a heading to go to it. You can also use the arrow keys to select a heading. Press Enter to go to that heading. Select **Hide** to close the outline. You can also use the outline shortcut.

Select **Recent** to see files that you opened before. Type in the recent-files field to find a file.

## Change the appearance

Select **Settings** to change the theme, font, font size, and shortcuts. You can select a font that is on your computer. You can also select the default monospace font. nia saves these settings for the next start.

On macOS and Linux, select **Transparent background** to see the desktop through the writing window. Use **Window opacity** to change how much of the desktop you can see.

Press Primary+Shift+J or select **Zen mode** in Settings to hide the footer and tab bar. Open panes, the outline, and search stay visible. Press Escape to show the controls again.

In Settings, select **Highlight current line** to show the active line with a different color. Select **Dim unfocused** to make other paragraphs or sentences dim.

## Use keyboard shortcuts

In this table, **Primary** means Cmd on macOS and Ctrl on Windows and Linux. You can change these shortcuts in Settings. Select **Restore defaults** to use the shortcuts for your operating system.

| Action | Shortcut |
| --- | --- |
| New document group | Primary+N or Primary+T |
| Open document | Primary+O |
| Save document | Primary+S; Ctrl+S also works on macOS |
| Find text in the active document | Primary+F; Ctrl+F also works on macOS |
| Find text in all tabs | Primary+Shift+F |
| Open or close the outline | Primary+Shift+O |
| Open recent files | Primary+P |
| Open Settings | Primary+, |
| Close the active document | Primary+W |
| Close the active tab group | Primary+Shift+W |
| Go to the next or previous split | Primary+] or Primary+[ |
| Select group 1 to 9 | Primary+1 to Primary+9 |
| Show one pane | Ctrl+Shift+L |
| Add a document in a pane on the right | Primary+D |
| Add a document in a pane below | Primary+Shift+D |
| Make the font larger or smaller | Primary++ or Primary+- |
| Add bold or italic markers | Primary+B or Primary+I |
| Undo or redo an edit | Primary+Z or Primary+Shift+Z |
| Start or stop Zen mode | Primary+Shift+J |

Primary+1 to Primary+9 are fixed shortcuts. Primary+9 selects the last group. Press Escape to close Settings, the Handbook, and search.

## Keep your work

nia looks for changes that other programs make to your files. If you have no unsaved changes, nia loads the new file text. If you have unsaved changes, nia keeps your text and offers **Save As**.

nia keeps unsaved drafts on your computer. After an interrupted session, nia opens these drafts in separate groups. Save each draft to keep it as a file.

If nia cannot save a file, it keeps your text in the editor. Read the error message. If the file changed, moved, or is not available, use **Save As** to save your text to a new file. If storage is full, make space or save to another place. nia tries again to save a recovery draft after a short wait. If a draft cannot be read at the next start, nia keeps it on disk and opens the other drafts.

nia keeps settings, recent file paths, and drafts on your computer. It does not write document text to logs. Read `PRIVACY.md` in the project or release archive for more information.
