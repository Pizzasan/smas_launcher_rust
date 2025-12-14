# very lazy win x64 rust fork/paste, all credits goes to stephini for the initial launcher idea and to snesrev for making smas playable on windows in the first place
> Super Mario World, Super Mario All-Stars, Mario, and all related names, assets, and trademarks are the property of Nintendo.
> This project is a fan-made launcher and is not affiliated with, endorsed, or approved by Nintendo.
> This launcher does not include any Nintendo game ROMs or copyrighted music. Users must supply their own legally obtained game files and music.


## Credits / Copyrights
- Font: Super Mario World  | Credit: David Fens 2013
- gilrs: https://github.com/Arvamer/gilrs
- SDL: https://github.com/libsdl-org/SDL
- SDL Image: https://github.com/libsdl-org/SDL_image
- SDL mixer: https://github.com/libsdl-org/SDL_mixer
- SDL ttf: https://github.com/libsdl-org/SDL_ttf
- serde-rs: https://github.com/serde-rs/serde
- SMW / SMAS: https://github.com/snesrev/smw
- SMAS Launcher (Python): https://github.com/stephini/SMAS_Launcher
> if i forgot any it wasn't on purpose




# Super Mario All-Stars Launcher for Super Mario World

This is a simple launcher for the Super Mario All-Stars branch of the Super Mario World port by snesrev. It allows you to easily run the game on your system. 

**Note: The releases and source code are updated in tandem. Please choose the option that suits you best, as both options provide the same functionality.**

## Prerequisites

Before using this launcher, make sure you have the following installed:

- Super Mario All-Stars ROM (smas.sfc)
- Super Mario World ROM (smw.sfc)

## Option 1: Using the Executable (Simpler and Cleaner)

Executable versions for Mac and Linux will be forthcoming.

1. Go to the "Releases" section of the repository.
2. Download the latest release.
3. Create a new folder and place the downloaded executable in it.
4. Copy the Super Mario All-Stars ROM (smas.sfc) and Super Mario World ROM (smw.sfc) into the same folder.
5. Run the executable file.
6. The launcher will launch the game, and you can start playing.

## Option 2: Using Python (Recommended for Python users)

[Python](https://www.python.org/downloads/) has tentative support for Mac and Linux. 

*Note: During the installation of Python make sure to check the box that adds Python to PATH.*

1. Download the source code from the repository.
2. Create a new folder and place the downloaded source code in it.
3. Copy the Super Mario All-Stars ROM (smas.sfc) and Super Mario World ROM (smw.sfc) into the same folder.
4. On Windows run the "Install-Dependencies-for-Windows.bat" file to download the required Python packages.
4. On Mac or Linux run the "Install-Dependencies-for-Mac-and-Linux.sh" file to download the required Python packages.
5. After downloading the dependencies, run the "launcher.pyw" script on windows or "Run on Mac and Linux.sh" on Mac and Linux.
6. The launcher will install the needed backend and then launch the game, and you can start playing.

## Troubleshooting

If you encounter any issues while using the launcher, please refer to the repository's issue tracker for known problems and solutions. You can also report new issues to help improve the launcher.

## Disclaimer

Please note that the usage of ROMs may infringe on copyright laws. Make sure you own the original game cartridges or have obtained the ROMs legally before using this launcher.

## Contributions

Contributions to this project are welcome. If you have any improvements or suggestions, feel free to submit a pull request or open an issue on the repository.

## License

This launcher is released under the [MIT License](LICENSE). Please refer to the license file for more information.
