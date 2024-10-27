# webfishing-midi
cross-platform midi player for the webfishing guitar!\
**Warning** ⚠️: the code may or may not be slop, I am not experienced with midi format

## Installation & Usage
Linux users may need additional runtime dependencies see [here](https://github.com/enigo-rs/enigo?tab=readme-ov-file#runtime-dependencies) and [here](https://github.com/nashaofu/xcap/?tab=readme-ov-file#linux-system-requirements)\
Windows users may need to install Microsoft [Visual C++ Redistributable](https://aka.ms/vs/17/release/vc_redist.x64.exe)
- Download the executable for your platform from [here](https://github.com/yobson1/webfishing-midi/releases)
- Place your midi files in the `./midi` directory next to the executable
- Run webfishing-midi
- Select a song by typing a name to search and/or using the arrow keys & enter to make a selection
- Tab over to the game and press backspace to start playing
- Press right shift to pause/resume playing
- Press escape to stop playing

### Interface
The program uses a simple terminal interface powered by [dialoguer](https://github.com/console-rs/dialoguer) you can select a midi by typing a name to search and using the arrow keys & enter to make a selection.

#### Track selection
When selecting a track you can use the arrow keys to navigate and space to select. Enter to confirm your selection.\
If a track has all of it's fields as "Unknown" it is likely a meta track that has no notes and just meta messages for things like tempo changes.

#### Demo
https://github.com/user-attachments/assets/c7b81e3e-f701-4470-bc7c-66a9a4e508da

## Supported platforms
As of now this has only been tested on Linux and Windows but I have taken care to use cross-platform libraries. If you encounter a problem please [open an issue](https://github.com/yobson1/webfishing-midi/issues) and I will try to resolve it

## Acknowledgements
- Got the note shifting idea/logic from [KevAquila](https://github.com/KevAquila/WEBFISHING-Guitar-Player) his code was used as reference
- Feature contributions from [Peacockli](https://github.com/Peacockli/webfishing-midi)
