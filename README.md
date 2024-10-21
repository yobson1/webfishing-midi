# webfishing-midi
cross-platform midi player for the webfishing guitar!\
**Warning** ⚠️: the code may or may not be slop, I am not experienced with midi format

## Installation & Usage
Linux users may need additional runtime dependencies see [here](https://github.com/enigo-rs/enigo?tab=readme-ov-file#runtime-dependencies) and [here](https://github.com/nashaofu/xcap/?tab=readme-ov-file#linux-system-requirements)
- Download the executable for your platform from [here](https://github.com/yobson1/webfishing-midi/releases)
- Place your midi files in the `./midi` directory next to the executable
- Run webfishing-midi
- Select a song by typing a name to search and/or using the arrow keys & enter to make a selection
- Tab over to the game and press backspace to start playing
- Press and hold ESCAPE to stop playing (this may take a moment to stop since it only checks for input when a note is played)

### Interface
The program uses a simple terminal interface powered by [dialoguer](https://github.com/console-rs/dialoguer) you can select a midi by typing a name to search and using the arrow keys & enter to make a selection.
### Skip overlapping strings
When you select a song you will be asked if you want to skip overlapping strings. This skips notes that would be played on the same string at the same time just with a different fret. This usually sounds pretty bad so you can choose to skip these notes and it will just play one note at a time on each string for each tick.
#### Demo
https://github.com/user-attachments/assets/c7b81e3e-f701-4470-bc7c-66a9a4e508da

## Supported platforms
As of now this has only been tested on Linux and Windows but I have taken care to use cross-platform libraries. If you encounter a problem please [open an issue](https://github.com/yobson1/webfishing-midi/issues) and I will try to resolve it

## Acknowledgements
- Got the note shifting idea/logic from [KevAquila](https://github.com/KevAquila/WEBFISHING-Guitar-Player) his code was used as reference
