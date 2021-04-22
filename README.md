# Arc Randomizer
## Prerequisites
- [ARCropolis 1.1.3 or higher](https://github.com/Raytwo/ARCropolis/releases/latest)
- [Skyline (Should come with ARCropolis)](https://github.com/skyline-dev/skyline/releases/tag/beta)

## Setup
Download the latest version of this plugin and put it in:
`sd:/atmosphere/contents/01006A800016E000/romfs/skyline/plugins`

## Usage
Create a folder called `Randomizer` in the following path:

`sd:/atmosphere/contents/01006A800016E000/romfs/`

Then replicate the ARC path for the file you want to randomize.

For Example:

Stream - `sd:/atmosphere/contents/01006A800016E000/romfs/Randomizer/stream;/sound/bgm/bgm_crs2_02_senjyou.nus3audio/*.nus3audios`
(Randomizes song anytime the song "Battlefield" is played)

Arc - `sd:/atmosphere/contents/01006A800016E000/romfs/Randomizer/ui/message/msg_bgm.msbt/*.msbt`
(Randomizes msbt file anytime msg_bgm.msbt is loaded)
