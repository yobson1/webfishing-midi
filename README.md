# webfishing-midi
cross-platform midi player for the webfishing guitar!\
**Warning** ⚠️: the code may or may not be slop, I am not experienced with midi format

## Installation & Usage
Linux users may need additional runtime dependencies see [here](https://github.com/enigo-rs/enigo?tab=readme-ov-file#runtime-dependencies) and [here](https://github.com/nashaofu/xcap/?tab=readme-ov-file#linux-system-requirements)
- Place your midi files in the `./midi` directory next to the executable
- Run webfishing-midi in a terminal

### Interface
The program uses a simple terminal interface powered by [dialoguer](https://github.com/console-rs/dialoguer) you can select a midi by typing a name to search and using the arrow keys & enter to make a selection.

## Supported platforms
As of now this has only been tested on Linux but I have taken care to use cross-platform libraries. If you encounter a problem please [open an issue](https://github.com/yobson1/webfishing-midi/issues) and I will try to resolve it

## Acknowledgements
- Got the note shifting idea/logic from [KevAquila](https://github.com/KevAquila/WEBFISHING-Guitar-Player) his code was used as reference
